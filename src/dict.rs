use lazy_static::lazy_static;
use std::collections::HashMap;
use std::pin::Pin;
use std::sync::{Arc, RwLock};
use log::error;
use std::sync::Mutex;
use crate::commands::COMMANDS;
use crate::connection::ConnectionHandler;
use crate::db::Db;
use crate::frame::Frame;
use crate::parse::Parse;

/// COMMAND_TABLE 存储所有命令的哈希表
/// `COMMAND_TABLE` stores a hash map of all commands.
lazy_static! {
    static ref COMMAND_TABLE: Arc<RwLock<HashMap<String, Command>>> = Arc::new(RwLock::new(HashMap::new()));
}

/// 创建命令的宏
/// A macro to create commands.
macro_rules! make_command {
    ($name:expr, $description:expr, $complexity:expr, $command_fn:expr) => {
        Command {
            name: $name.to_string(),
            description: $description.to_string(),
            time_complexity: $complexity.to_string(),
            command_fn: Arc::new($command_fn),
        }
    };
}

/// Command 命令细节
/// `Command` struct containing the details of a command.
#[derive(Clone)]
pub struct Command {
    pub name: String,
    pub command_fn: Arc<dyn Fn(&mut Arc<Mutex<Db>>,&mut Parse) -> crate::Result<Frame> + Send + Sync + 'static>,
    pub time_complexity: String,
    pub description: String,
}

impl Command {
    /// 加载所有命令到内存命令表中
    /// Load all commands into the in-memory command table.
    pub fn load_commands() {
        // 遍历所有命令元数据，并将其插入到命令表中
        // Iterate over all command metadata and insert them into the command table.
        let mut command_map = match COMMAND_TABLE.write() {
            Ok(lock) => lock,
            Err(poisoned) => {
                // 锁被污染时的处理方式
                // Handle the case when the lock is poisoned.
                error!("无法获得锁: {:?}", poisoned);
                return;
            }
        };
        for &(name, description, time_complexity, command_fn) in COMMANDS.iter() {
            let command = make_command!(name, description, time_complexity, command_fn);
            command_map.insert(command.name.clone(), command);
        }
    }

    /// 获取命令对应的处理函数
    /// Get the corresponding handler function for the command.
    pub fn get_command_fn(name: &str)
                          -> Option<Arc<dyn Fn(&mut Arc<Mutex<Db>>, &mut Parse) -> crate::Result<Frame> + Send + Sync + 'static>> {
        if name.is_empty() {
            return None;
        }

        // 尝试获取读锁
        // Try to acquire a read lock.
        let command_map = match COMMAND_TABLE.read() {
            Ok(lock) => lock,
            Err(poisoned) => {
                // 锁被污染时的处理方式
                // Handle the case when the lock is poisoned.
                error!("无法获得锁: {:?}", poisoned);
                return None;
            }
        };

        // 获取命令并返回处理函数
        // Get the command and return the handler function.
        let command = command_map.get(&name.to_lowercase());
        command.map(|cmd| cmd.command_fn.clone())
    }

    /// 获取命令详情
    /// Get the details of the command.
    pub fn get_command_detail(name: &str) -> Option<Command> {
        if name.is_empty() {
            return None;
        }
        // 尝试获取读锁
        // Try to acquire a read lock.
        let command_map = match COMMAND_TABLE.read() {
            Ok(lock) => lock,
            Err(poisoned) => {
                // 锁被污染时的处理方式
                // Handle the case when the lock is poisoned.
                error!("无法获得锁: {:?}", poisoned);
                return None;
            }
        };
        let command = command_map.get(&name.to_lowercase());
        // 如果命令存在，返回命令的克隆
        // If the command exists, return a clone of the command.
        command.map(|cmd| cmd.clone())
    }

    /// 检查命令是否存在
    /// Check if the command exists.
    pub fn exists(name: &str) -> bool {
        if name.is_empty() {
            return false;
        }
        // 获取读锁并检查命令是否存在
        // Acquire a read lock and check if the command exists.
        let command_map = match COMMAND_TABLE.read() {
            Ok(lock) => lock,
            Err(poisoned) => {
                // 锁被污染时的处理方式
                // Handle the case when the lock is poisoned.
                error!("无法获得锁: {:?}", poisoned);
                return false;
            }
        };
        command_map.contains_key(&name.to_lowercase())
    }
}
