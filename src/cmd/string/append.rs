use std::sync::Arc;
use std::sync::Mutex;
use crate::connection::ConnectionHandler;
use crate::db::{Db, DbType};
use crate::frame::Frame;
use crate::parse::Parse;
use crate::persistence::aof::propagate_aof;

/// `Append` command for string type.
/// `Append` 命令用于字符串类型。
///
/// Appends the specified value to the string value of the key.
/// If the key does not exist, a new key is created with the specified value.
/// 返回追加后的新字符串的长度。
pub struct Append {
    key: String,  // The key to append the value to. / 需要追加值的键
    value: String,  // The value to append. / 要追加的值
}

impl Append {
    /// Executes the `append` command.
    /// 执行 `append` 命令。
    ///
    /// # Arguments
    /// # 参数
    /// - `db`: Shared reference to the database for access. / 用于访问数据库的共享引用。
    /// - `parse`: For parsing the command from the client. / 用于解析客户端传来的命令。
    ///
    /// # Return
    /// # 返回
    /// Returns the length of the new string after the append operation. / 返回追加后的新字符串的长度。
    pub fn append_command(
        db: &mut Arc<Mutex<Db>>,
        parse: &mut Parse,
    ) -> crate::Result<Frame> {
        match Append::parse_command(parse) {
            Ok(append) => {
                let mut db = db.lock().unwrap();
                // Get the current value of the key
                // 获取键当前的值
                let current_value = db.get(&append.key);

                let new_value = match current_value {
                    // If the key exists, append the new value
                    // 如果键存在，追加新的值
                    Some(DbType::String(existing_value)) => {
                        format!("{}{}", existing_value, append.value)
                    },
                    // If the key does not exist, set the new value
                    // 如果键不存在，设置新的值
                    _ => append.value.clone(),
                };

                // Set or update the value of the key
                // 设置或更新键的值
                db.set(&append.key, DbType::String(new_value.clone()), None);


                // Return the length of the new string
                // 返回追加后的新值的长度
                Ok(Frame::Integer(new_value.len() as i64))
            }
            Err(_) => {
                // If command parsing fails, return an error message
                // 如果命令解析失败，返回错误信息
                Ok(Frame::Error("ERR wrong number of arguments for 'append' command".to_string()))
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
    /// Returns the parsed `Append` instance containing the key and value. / 返回解析后的 `Append` 实例，包含键和值。
    fn parse_command(parse: &mut Parse) -> crate::Result<Self> {
        let key = parse.next_string()?;  // Get the key from the command. / 从命令中获取键
        let value = parse.next_string()?;  // Get the value from the command. / 从命令中获取值

        Ok(Append { key, value })
    }
}
