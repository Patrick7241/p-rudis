use std::sync::Arc;
use std::sync::Mutex;
use crate::db::{Db, DbType};
use crate::frame::Frame;
use crate::parse::Parse;

/// `Decr` command for string type.
/// `Decr` 命令用于字符串类型。
///
/// Decreases the numeric value of the specified key by 1.
/// If the key does not exist, a new key is created with the value -1.
/// 返回减少后的新值。
pub struct Decr {
    key: String,  // The key whose value will be decreased. / 要减少数值的键
}

impl Decr {
    /// Executes the `decr` command.
    /// 执行 `decr` 命令。
    ///
    /// # Arguments
    /// # 参数
    /// - `db`: Shared reference to the database for access. / 用于访问数据库的共享引用。
    /// - `parse`: For parsing the command from the client. / 用于解析客户端传来的命令。
    ///
    /// # Return
    /// # 返回
    /// Returns the new value after decrementing. / 返回减少后的新值。
    pub fn decr_command(
        db: &mut Arc<Mutex<Db>>,
        parse: &mut Parse
    ) -> crate::Result<Frame> {
        match Decr::parse_command(parse) {
            Ok(decr) => {
                let mut db = db.lock().unwrap();
                // Get the current value of the key
                // 获取键的当前值
                match db.get(&decr.key) {
                    // If the key exists and its value is a number, decrement it
                    // 如果键存在且值为数字，进行减少
                    Some(DbType::String(value)) => {
                        match value.parse::<i64>() {  // Allow negative values
                            Ok(current_value) => {
                                let new_value = current_value - 1; // Decrease by 1 / 减少 1
                                db.set(&decr.key, DbType::String(new_value.to_string()), None);
                                Ok(Frame::Integer(new_value))  // Return the new value / 返回新值
                            }
                            // If the value is not a number, return an error
                            // 键的值不是数字，返回错误
                            Err(_) => {
                                Ok(Frame::Error("ERR value is not an integer or out of range".to_string()))
                            }
                        }
                    }
                    // If the key doesn't exist, initialize it as -1 and then decrease
                    // 如果键不存在，初始化为 -1，然后减少
                    _ => {
                        let new_value = -1;  // Initialize with -1 / 初始化为 -1
                        db.set(&decr.key, DbType::String(new_value.to_string()), None);
                        Ok(Frame::Integer(new_value))  // Return the new value / 返回新值
                    }
                }
            }
            Err(_) => {
                // If command parsing fails, return an error message
                // 如果命令解析失败，返回错误信息
                Ok(Frame::Error("ERR wrong number of arguments for 'decr' command".to_string()))
            }
        }
    }

    /// Parses the command and retrieves the parameters.
    /// 解析命令并获取参数。
    ///
    /// # Arguments
    /// # 参数
    /// - `parse`: The `Parse` instance used to parse the command. / 用于解析命令的 `Parse` 实例。
    ///
    /// # Return
    /// # 返回
    /// Returns the parsed `Decr` instance containing the key. / 返回解析后的 `Decr` 实例，包含键。
    fn parse_command(parse: &mut Parse) -> crate::Result<Self> {
        let key = parse.next_string()?;  // Get the key from the command. / 从命令中获取键
        Ok(Decr { key })
    }
}
