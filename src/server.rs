use std::io::Error;
use std::process::id;
use log::{error, info};
use tokio::net::TcpListener;
use crate::parse;
use crate::connection::ConnectionHandler;
use crate::db::{Db, DbHolder};
use crate::shutdown::Shutdown;
use crate::dict::Command;

#[derive(Debug)]
pub struct Listener {
    /// 监听客户端的连接
    listener: TcpListener,
    /// 管理数据库
    db_holder: DbHolder
}

#[derive(Debug)]
pub struct Handler{
    /// 存储数据库
    db:Db,
    /// 客户端连接
    connection:ConnectionHandler,
    /// 关闭信号
    shutdown:Shutdown
}

fn go(){
    let pid = id();
    let welcome = format!(
        r#"
        / \__
       (    @\___
       /         O
      /   (_____ /
     /_____/   U
    {}
    PID: {}
    "#,"欢迎使用p-rudis",pid);
    println!("{}", welcome);
}


/// 启动 p-rudis 服务端
pub async fn run(listener: TcpListener,shutdown: impl Future){
    // 启动界面
    go();

    let mut listener=Listener{
        listener,
        db_holder:DbHolder::new()
    };
   tokio::select! {
       res=listener.run()=>{
           if let Err(err)=res{
               error!("监听出错: {}",err)
           }
       },
       _=shutdown=>{
           info!("接收到关闭信号，服务端已优雅关闭")
       }
   }
}
impl Listener {
    /// 启动监听
    async fn run(&mut self)->Result<(),Error>{
        loop {
            // 接收连接
            let ( socket,addr)=self.listener.accept().await?;
            info!("接收客户端连接: {}",addr);
            // 处理连接
            let mut handler=Handler{
                db:self.db_holder.clone(),
                connection:ConnectionHandler::new(socket),
                shutdown:Shutdown::new()
            };
            tokio::spawn(async move {
                if let Err(err)=handler.run().await{
                    error!("处理连接: {}",err)
                }
            });
            }
        }
}

impl Handler{
    async fn run(&mut self)->crate::Result<()>{
        while !self.shutdown.is_shutdown() {
            if let Err(err) = self.process_data().await {
                // TODO 这里不应该暂停，应该发送错误信息回客户端，待修改
                Err(err)?;
                continue
            }
        }
        Ok(())
    }
    /// 读取和解析数据
    async fn process_data(&mut self) ->crate::Result<()> {
        // 读取数据并处理错误
        let data = self.connection.read_data().await?;
        // 解析数据并处理错误
        let mut parts =parse::Parse::new(data)?;
        // 获取命令名称并转换为小写
        let command_name=parts.next_string()?.to_lowercase();
        // 查看命令是否存在于命令表中
        if !Command::exists(&command_name){
            self.connection.write_data(format!("不存在{}命令！", command_name)).await?;
            return Err(Box::new(Error::new(std::io::ErrorKind::Other, "命令不存在")));
        }

        Ok(())
    }

}


