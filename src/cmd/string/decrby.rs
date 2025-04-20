use std::sync::Arc;
use std::sync::Mutex;
use crate::db::{Db, DbType};
use crate::frame::Frame;
use crate::parse::Parse;

/// `DecrBy` command for string type.
/// `DecrBy` 命令用于字符串类型。
///
/// Decreases the numeric value of the specified key by the given step.
/// If the key does not exist, a new key is created with the value -step.
/// 返回减少后的新值。
pub struct DecrBy {
    key: String,  // The key whose value will be decreased. / 要减少数值的键
    step: i64,    // The step by which the value will be decreased. / 减少值的步长
}

impl DecrBy {
    /// Executes the `decrby` command.
    /// 执行 `decrby` 命令。
    ///
    /// # Arguments
    /// # 参数
    /// - `db`: Shared reference to the database for access. / 用于访问数据库的共享引用。
    /// - `parse`: For parsing the command from the client. / 用于解析客户端传来的命令。
    ///
    /// # Return
    /// # 返回
    /// Returns the new value after decrementing. / 返回减少后的新值。
    pub fn decrby_command(
        db: &mut Arc<Mutex<Db>>,
        parse: &mut Parse
    ) -> crate::Result<Frame> {
        match DecrBy::parse_command(parse) {
            Ok(decr) => {
                let mut db = db.lock().unwrap();
                // Get the current value of the key
                // 获取键的当前值
                match db.get(&decr.key) {
                    // If the key exists and its value is a number, decrement it by the step
                    // 如果键存在且值为数字，按步长减少
                    Some(DbType::String(value)) => {
                        match value.parse::<i64>() {
                            Ok(current_value) => {
                                let new_value = current_value - decr.step;  // Decrease by step / 按步长减少
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
                    // If the key doesn't exist, initialize it with -step and then decrease
                    // 如果键不存在，初始化为 -step，然后减少
                    _ => {
                        let new_value = -decr.step;  // Initialize with -step / 初始化为 -step
                        db.set(&decr.key, DbType::String(new_value.to_string()), None);
                        Ok(Frame::Integer(new_value))  // Return the new value / 返回新值
                    }
                }
            }
            Err(_) => {
                // If command parsing fails, return an error message
                // 如果命令解析失败，返回错误信息
                Ok(Frame::Error("ERR wrong number of arguments for 'decrby' command".to_string()))
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
    /// Returns the parsed `DecrBy` instance containing the key and step. / 返回解析后的 `DecrBy` 实例，包含键和值。
    fn parse_command(parse: &mut Parse) -> crate::Result<Self> {
        let key = parse.next_string()?;  // Get the key from the command. / 从命令中获取键
        let step = parse.next_string()?;  // Get the step value from the command. / 从命令中获取步长值

        // Convert step to i64 type
        // 把step转成i64类型
        // If conversion fails, return an error / 若转化失败返回错误
        let step: i64 = match step.parse() {
            Ok(num) => num,
            Err(_) => return Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, "ERR value is not an integer or out of range"))),
        };

        Ok(DecrBy { key, step })
    }
}
