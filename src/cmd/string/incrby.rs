use std::sync::Arc;
use std::sync::Mutex;
use crate::db::{Db, DbType};
use crate::frame::Frame;
use crate::parse::Parse;

/// `IncrBy` command for string type.
/// `IncrBy` 命令用于字符串类型。
///
/// Increases the value of the specified key by a given step. No default value.
/// 将指定键的数值增加指定的步长，无默认值。
/// If the key does not exist, a new key is created with the value of the step.
/// 如果键不存在，新建一个键，值为步长。
/// Returns the new value after the increment.
/// 返回增加后的新值。
pub struct IncrBy {
    key: String,  // The key to increase the value for / 要增加值的键
    step: i64,    // The increment step / 步长
}

impl IncrBy {
    /// Executes the `incrby` command.
    /// 执行 `incrby` 命令。
    ///
    /// # Arguments
    /// # 参数
    /// - `db`: Shared reference to the database for access. / 用于访问数据库的共享引用。
    /// - `parse`: For parsing the command from the client. / 用于解析客户端传来的命令。
    ///
    /// # Return
    /// # 返回
    /// - If the key exists and its value is a number, increase it by the specified step and return the new value. / 如果键存在且值为数字，按指定的步长增加并返回新值。
    /// - If the key does not exist, initialize it to the step value and return the new value. / 如果键不存在，将其初始化为步长的值并返回新值。
    pub fn incrby_command(
        db: &mut Arc<Mutex<Db>>,
        parse: &mut Parse
    ) -> crate::Result<Frame> {
        match IncrBy::parse_command(parse) {
            Ok(incr) => {
                let mut db = db.lock().unwrap();
                // Get the current value of the key
                // 获取键的当前值
                match db.get(&incr.key) {
                    // If the key exists and its value is a number, increase it by the step
                    // 如果键存在且值为数字，按步长增加
                    Some(DbType::String(value)) => {
                        match value.parse::<i64>() {
                            Ok(current_value) => {
                                let new_value = current_value + incr.step;
                                db.set(&incr.key, DbType::String(new_value.to_string()), None);
                                Ok(Frame::Integer(new_value))
                            }
                            // If the value is not a number, return an error
                            // 如果值不是数字，返回错误
                            Err(_) => {
                                Ok(Frame::Error("ERR value is not an integer or out of range".to_string()))
                            }
                        }
                    }
                    // If the key does not exist, initialize it to the step value and return the new value
                    // 如果键不存在，将其初始化为步长的值并返回新值
                    _ => {
                        let new_value = incr.step;
                        db.set(&incr.key, DbType::String(new_value.to_string()), None);
                        Ok(Frame::Integer(new_value))
                    }
                }
            }
            // If the command has an incorrect number of arguments, return an error
            // 如果命令参数个数不正确，返回错误
            Err(_) => {
                Ok(Frame::Error("ERR wrong number of arguments for 'incrby' command".to_string()))
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
    /// Returns the parsed `IncrBy` instance containing the key and step. / 返回解析后的 `IncrBy` 实例，包含键和步长。
    fn parse_command(parse: &mut Parse) -> crate::Result<Self> {
        let key = parse.next_string()?;  // Get the key to be incremented / 获取要增加的键
        let step = parse.next_string()?;  // Get the step value / 获取步长值

        // Convert the step value to i64 type
        // 将步长值转换为 i64 类型
        let step: i64 = match step.parse() {
            Ok(num) => num,
            Err(_) => return Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, "ERR value is not an integer or out of range"))),
        };

        Ok(IncrBy { key, step })
    }
}
