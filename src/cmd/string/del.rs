use std::sync::Arc;
use std::sync::Mutex;
use crate::db::Db;
use crate::frame::Frame;
use crate::parse::Parse;

/// `Del` command for string type.
/// `Del` 命令用于字符串类型。
///
/// Deletes the specified keys and returns the number of keys that were deleted.
/// 删除指定的多个键，返回成功删除的键的数量
pub struct Del {
    keys: Vec<String>,  // List of keys to be deleted. / 要删除的键的列表
}

impl Del {
    /// Executes the `del` command.
    /// 执行 `del` 命令。
    ///
    /// # Arguments
    /// # 参数
    /// - `db`: Shared reference to the database for access. / 用于访问数据库的共享引用。
    /// - `parse`: For parsing the command from the client. / 用于解析客户端传来的命令。
    ///
    /// # Return
    /// # 返回
    /// Returns the number of keys that were deleted. / 返回成功删除的键的数量。
    pub fn del_command(
        db: &mut Arc<Mutex<Db>>,
        parse: &mut Parse
    ) -> crate::Result<Frame> {
        match Del::parse_command(parse) {
            Ok(del) => {
                let mut db = db.lock().unwrap();
                let mut deleted_count = 0;

                // Iterate through all the keys and try to delete them
                // 遍历所有键，尝试删除它们
                for key in del.keys {
                    if db.del(&key) {
                        deleted_count += 1;  // If the key is deleted, increment the count / 如果键被删除，增加计数
                    }
                }

                // Return the number of deleted keys
                // 返回删除的键的数量
                Ok(Frame::Integer(deleted_count))
            }
            Err(_) => {
                // If parsing fails, return an error message
                // 如果命令解析失败，返回错误信息
                Ok(Frame::Error("ERR wrong number of arguments for 'del' command".to_string()))
            }
        }
    }

    /// Validates the command and retrieves the parameters.
    /// 验证命令是否合法，并获取命令参数。
    ///
    /// # Arguments
    /// # 参数
    /// - `parse`: The `Parse` instance used to parse the command. / 用于解析命令的 `Parse` 实例。
    ///
    /// # Return
    /// # 返回
    /// Returns the parsed `Del` instance containing the keys to delete. / 返回解析后的 `Del` 实例，包含要删除的键。
    fn parse_command(parse: &mut Parse) -> crate::Result<Self> {
        let mut keys = Vec::new();

        // Keep parsing until there are no more keys
        // 一直到没有更多的键为止
        while let Ok(key) = parse.next_string() {
            keys.push(key);
        }

        // If no keys were provided, return an error
        // 如果没有传入任何键，返回错误
        if keys.is_empty() {
            return Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                "ERR wrong number of arguments for 'del' command"
            )));
        }

        Ok(Del { keys })
    }
}
