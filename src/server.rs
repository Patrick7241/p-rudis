use std::io::Error;
use std::ops::Deref;
use std::process::id;
use std::sync::{Arc};
use log::{error, info};
use tokio::net::{TcpListener, TcpStream};
use std::sync::Mutex;
use tokio::select;
use tokio::sync::broadcast;
use crate::{cmd, dict, frame, parse};
use crate::connection::ConnectionHandler;
use crate::db::{Db, DbHolder};
use crate::shutdown::Shutdown;
use crate::dict::Command;
use crate::frame::Frame;
use crate::persistence::aof::load_aof;

#[derive(Debug)]
pub struct Listener {
    /// 监听客户端的连接
    /// Listen to client connections
    listener: TcpListener,
    /// 管理数据库
    /// Manage the database
    db_holder: DbHolder,
    /// 用于处理发布订阅模式的关闭信号
    /// Used to handle the shutdown signal for the pub/sub model
    notify_shutdown: broadcast::Sender<()>,
}

#[derive(Debug)]
pub struct Handler {
    /// 存储数据库
    /// Store the database
    db: Arc<Mutex<Db>>,
    /// 客户端连接
    /// Client connection
    connection: ConnectionHandler,
    /// 关闭信号
    /// Shutdown signal
    shutdown: Shutdown,
}

// TODO port硬编码待修改
// TODO: hard-coded port needs to be changed
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
/// Start the p-rudis server
pub async fn run(listener: TcpListener, shutdown: impl Future) {
    // 启动界面
    // Start the interface
    go();

    let mut listener = Listener {
        listener,
        db_holder: DbHolder::new(),
        notify_shutdown: broadcast::channel(1).0,
    };
    select! {
        res = listener.run() => {
            if let Err(err) = res {
                error!("错误: {}", err)  // Error listening
            }
        },
        _ = shutdown => {
            info!("接收到关闭信号，服务端已优雅关闭")  // Shutdown signal received, gracefully closing the server
        }
    }
}

impl Listener {
    /// 启动监听
    /// Start listening
    async fn run(&mut self) -> Result<(), Error> {
        if let Ok((time, _)) = load_aof(&mut self.db_holder.get_db(), "test.aof").await {
            // 成功加载 AOF 数据后处理
           info!("加载 AOF 数据花费时间: {} 毫秒", time);
        } else {
            error!("加载 AOF 数据失败");
        }
        loop {
            // 接收连接
            // Accept a connection
            let (socket, addr) = self.listener.accept().await?;
            info!("接收客户端连接: {}", addr);  // Accepting client connection
            // 处理连接
            // Handle the connection
            let mut handler = Handler {
                db: self.db_holder.get_db(),
                connection: ConnectionHandler::new(Arc::new(tokio::sync::Mutex::new(socket))),
                shutdown: Shutdown::new(self.notify_shutdown.subscribe()),
            };

           tokio::spawn(async move {
                if let Err(err) = handler.run().await {
                    error!("处理连接: {}", err)  // Error handling connection
                }
            });
        }
    }
}

impl Handler {
    async fn run(&mut self) -> crate::Result<()> {
        while !self.shutdown.is_shutdown() {
            // TODO 处理关闭后的逻辑，保存数据等
            // TODO: Handle post-shutdown logic, such as saving data
            if let Err(err) = self.process_data().await {
                Err(err)?;  // Error processing data
                continue;
            }
        }
        Ok(())
    }

    /// 读取和解析数据
    /// Read and parse data
    async fn process_data(&mut self) -> crate::Result<()> {
        // 读取数据并处理错误
        // Read data and handle errors
        let data = self.connection.read_data().await?;
        // 解析数据并处理错误
        // Parse data and handle errors
        let mut parts = parse::Parse::new(data)?;
        // 获取命令名称并转换为小写
        // Get the command name and convert it to lowercase
        let command_name = parts.next_string()?.to_lowercase();
        // 查看命令是否存在于命令表中
        // Check if the command exists in the command table
        if !Command::exists(&command_name) {
            self.connection
                .write_data(Frame::Error(format!("ERR unknown command '{}'", command_name)))
                .await?;  // Write error if command is unknown
        } else {
            // 命令存在，获取并调用对应处理函数
            // If command exists, get and call the corresponding handler function
            if let Some(command_fn) = Command::get_command_fn(&command_name) {
                // TODO 对于需要阻塞返回的函数暂时单独处理，后续可以封装一个阻塞处理的命令表
                // TODO: Temporarily handle blocking return functions, later can encapsulate a blocking command table
                match command_name.as_str() {
                    "subscribe"=>{
                        cmd::pubsub::subscribe::Subscribe::subscribe_command(&mut self.db, &mut parts, &mut self.connection, &mut self.shutdown)
                            .await?;  // Handle subscribe command
                        return Ok(());
                    }
                   "psubscribe"=>{
                    cmd::pubsub::psubscribe::PSubscribe::psubscribe_command(&mut self.db, &mut parts, &mut self.connection, &mut self.shutdown)
                        .await?;
                    return Ok(());
                    }
                    _=>{
                        // 传数据库，Parse命令内容,返回错误信息
                        // Pass the database, parse the command content, return error information
                        let res = command_fn(&mut self.db, &mut parts)?;
                        self.connection.write_data(res).await?;  // Write result to connection
                    }
                }
            } else {
                // 处理错误
                // Handle error
                self.connection
                    .write_data(Frame::Error(format!("ERR unknown command '{}'", command_name)))
                    .await?;  // Write error if command is unknown
            }
        }
        Ok(())
    }
}
