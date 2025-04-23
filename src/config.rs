use serde::Deserialize;
use std::{env, fs};
use lazy_static::lazy_static;
use std::sync::RwLock;
use tokio::io;

// 配置结构体
#[derive(Debug, Clone, Deserialize)]
pub struct AofConfig {
    pub enabled: bool,          // 是否启用AOF
    pub appendfsync: u64,       // AOF写入间隔时间，以秒为单位
    pub file_path: String,      // AOF文件存储位置
}

#[derive(Debug, Clone, Deserialize)]
pub struct RdbConfig {
    pub enabled: bool,          // 是否启用RDB
    pub save_interval: u64,     // RDB保存间隔时间，以秒为单位
    pub file_path: String,      // RDB文件存储位置
}

#[derive(Debug, Clone, Deserialize)]
pub struct ServerConfig {
    pub address: String,        // 服务端地址
}

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub aof: AofConfig,         // AOF配置
    pub rdb: RdbConfig,         // RDB配置
    pub server: ServerConfig,   // 服务端配置
}

// 使用 lazy_static 和 RwLock 定义全局可变配置
lazy_static! {
    pub static ref CONFIG: RwLock<Config> = RwLock::new(Config {
        aof: AofConfig {
            enabled: false,
            appendfsync: 0,
            file_path: String::new(),
        },
        rdb: RdbConfig {
            enabled: false,
            save_interval: 0,
            file_path: String::new(),
        },
        server: ServerConfig {
            address: String::new(),
        },
    });
}

// 解析配置文件并将结果存储到全局变量
pub fn parse_config(file_path: &str) -> io::Result<()> {
    // 读取文件内容
    let contents = fs::read_to_string(file_path)?;

    // 解析 TOML 内容到 Config 结构体
    let config: Config = toml::de::from_str(&contents).map_err(|e| {
        io::Error::new(io::ErrorKind::InvalidData, format!("Failed to parse TOML: {}", e))
    })?;

    // 更新全局配置
    let mut config_lock = CONFIG.write().unwrap();
    *config_lock = config;

    Ok(())
}

// 获取全局 AOF 配置
pub fn get_aof_config() -> AofConfig {
    let config_lock = CONFIG.read().unwrap();
    config_lock.aof.clone()
}

// 获取全局 RDB 配置
pub fn get_rdb_config() -> RdbConfig {
    let config_lock = CONFIG.read().unwrap();
    config_lock.rdb.clone()
}

// 获取全局服务端配置
pub fn get_server_config() -> ServerConfig {
    let config_lock = CONFIG.read().unwrap();
    config_lock.server.clone()
}
