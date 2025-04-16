use tokio::net::TcpListener;
use tokio::signal;
use p_rudis::Result;

mod server;
mod connection;
mod commands;
mod db;
mod frame;
mod pubsub;
mod shutdown;
mod log;
mod dict;
mod parse;
mod cmd;
// 目前只编写并启用服务端
#[tokio::main]
async fn main()->Result<()> {
    // 初始化日志服务
    log::init::setup_logger()?;
    // 从文件中加载所有指令到内存，key是命令名，value是命令细节信息
    dict::Command::load_commands();
    // TODO 改成参数启动或者配置文件启动
    let listener = TcpListener::bind("127.0.0.1:6379").await?;
    server::run(listener,signal::ctrl_c()).await;
    Ok(())
}
