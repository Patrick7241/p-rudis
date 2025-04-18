use std::io::Error;
use std::process::id;
use std::sync::{Arc};
use log::{error, info};
use tokio::net::{TcpListener, TcpStream};
use std::sync::Mutex;
use crate::{dict, frame, parse};
use crate::connection::ConnectionHandler;
use crate::db::{Db, DbHolder};
use crate::shutdown::Shutdown;
use crate::dict::Command;
use crate::frame::Frame;

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
    db:Arc<Mutex<Db>>,
    /// 客户端连接
    connection:ConnectionHandler,
    /// 关闭信号
    shutdown:Shutdown
}

// TODO port硬编码待修改
fn go() {
    let pid = id();
    let welcome = format!(
        r#"
        / \__                 欢迎使用p-rudis
       (    @\___
       /         O
      /   (_____ /            PORT: 6379
     /_____/   U              PID: {}
    "#,
        pid
    );
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
                db:self.db_holder.get_db(),
                connection:ConnectionHandler::new(Arc::new(tokio::sync::Mutex::new(socket))),
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
            // TODO 处理关闭后的逻辑，保存数据等
            if let Err(err) = self.process_data().await {
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
            self.connection
                .write_data(Frame::Error(format!("ERR unknown command '{}'", command_name)))
                .await?;
        }else {
            // 命令存在，获取并调用对应处理函数
            if let Some(command_fn) = Command::get_command_fn(&command_name) {
                // 传数据库，connection连接，Parse命令内容，返回错误信息和发送会客户端的信息
                let res = command_fn(&mut self.db, &mut parts)?;
                self.connection.write_data(res).await?;
            } else {
                // 处理错误
                self.connection
                    .write_data(Frame::Error(format!("ERR unknown command '{}'", command_name)))
                    .await?;
            }
        }
        Ok(())
    }

}


