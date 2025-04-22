use std::sync::{Arc, Mutex};
use crate::db::Db;
use crate::frame::Frame;
use crate::parse::Parse;
use crate::persistence::aof::propagate_aof;

/// `Del` command for deleting keys.
/// `Del` 命令用于删除键。
pub struct Del {
    keys: Vec<String>,  // List of keys to be deleted. / 要删除的键的列表
}

impl Del {
    /// Executes the `DEL` command to delete the specified keys.
    /// 执行 `DEL` 命令删除指定的键。
    ///
    /// # Arguments
    /// - `db`: Shared reference to the database for access. / 用于访问数据库的共享引用。
    /// - `parse`: For parsing the command from the client. / 用于解析客户端传来的命令。
    ///
    /// # Return
    /// Returns the number of keys that were deleted. / 返回成功删除的键的数量。
    pub fn del_command(db: &mut Arc<Mutex<Db>>, parse: &mut Parse) -> crate::Result<Frame> {
        match Del::parse_command(parse) {
            Ok(del) => {
                let mut db = db.lock().unwrap();
                let mut deleted_count = 0;

                // Iterate through all the keys and attempt to delete them
                // 遍历所有键，尝试删除它们
                for key in del.keys {
                    if db.del(&key) {
                        // Propagate AOF for each deletion
                        // 删除后传播到 AOF
                        Del::propagate_aof("del", &key);
                        deleted_count += 1;  // Increment the count of deleted keys / 增加删除的键计数
                    }
                }

                // Return the number of deleted keys
                // 返回删除的键的数量
                Ok(Frame::Integer(deleted_count))
            }
            Err(_) => {
                // Return error if parsing fails / 解析失败时返回错误
                Ok(Frame::Error("ERR wrong number of arguments for 'del' command".to_string()))
            }
        }
    }

    /// Parses the `DEL` command and retrieves the keys to delete.
    /// 解析 `DEL` 命令并获取要删除的键。
    ///
    /// # Arguments
    /// - `parse`: The `Parse` instance used to parse the command. / 用于解析命令的 `Parse` 实例。
    ///
    /// # Return
    /// Returns the parsed `Del` instance containing the keys to delete. / 返回解析后的 `Del` 实例，包含要删除的键。
    fn parse_command(parse: &mut Parse) -> crate::Result<Self> {
        let mut keys = Vec::new();

        // Parse all keys until no more keys are available
        // 解析所有键，直到没有更多键
        while let Ok(key) = parse.next_string() {
            keys.push(key);
        }

        // If no keys were provided, return an error
        // 如果没有传入任何键，返回错误
        if keys.is_empty() {
            return Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                "ERR wrong number of arguments for 'del' command",
            )));
        }

        Ok(Del { keys })
    }

    /// Propagates the `DEL` command to AOF.
    /// 将 `DEL` 命令传播到 AOF。
    fn propagate_aof(command: &str, key: &str) {
        propagate_aof(command.to_string(), vec![key.to_string()]);
    }
}
