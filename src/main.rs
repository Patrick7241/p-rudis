use tokio::net::TcpListener;
use tokio::signal;
use p_rudis::Result;

mod server;
mod connection;
mod command;
mod db;
mod frame;
mod pubsub;
mod shutdown;
mod util;
mod log;

// 目前只编写并启用服务端
#[tokio::main]
async fn main()->Result<()> {
    // 初始化日志服务
    log::init::setup_logger()?;
    // TODO 改成参数启动或者配置文件启动
    let listener = TcpListener::bind("127.0.0.1:6379").await?;
    server::run(listener,signal::ctrl_c()).await;
    Ok(())
}
