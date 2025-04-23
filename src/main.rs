use tokio::net::TcpListener;
use tokio::signal;
use p_rudis::config::{get_aof_config, get_server_config, parse_config};
use p_rudis::Result;
use p_rudis::log;
use p_rudis::dict;
use p_rudis::server;

// 目前只编写并启用服务端
#[tokio::main]
async fn main()->Result<()> {
    // 解析配置文件
    parse_config("src/config.toml")?;
    // 初始化日志服务
    log::init::setup_logger()?;
    // 从文件中加载所有指令到内存，key是命令名，value是命令细节信息
    dict::Command::load_commands();
    // 获取启动参数
    let server_config = get_server_config();
    let listener = TcpListener::bind(server_config.address).await?;
    server::run(listener,signal::ctrl_c()).await;
    Ok(())
}

