use std::sync::Arc;
use std::sync::Mutex;
use crate::db::{Db, DbType};
use crate::frame::Frame;
use crate::parse::Parse;

/// `Get` command for string type.
/// `Get` 命令用于字符串类型。
///
/// Retrieves the value of the specified key.
/// 获取指定键的值。
#[derive(Debug)]
pub struct Get {
    key: String,  // The key to get the value for / 要获取值的键
}

impl Get {
    /// Executes the `get` command.
    /// 执行 `get` 命令。
    ///
    /// # Arguments
    /// # 参数
    /// - `db`: Shared reference to the database for access. / 用于访问数据库的共享引用。
    /// - `parse`: For parsing the command from the client. / 用于解析客户端传来的命令。
    ///
    /// # Return
    /// # 返回
    /// - If the key exists and its value is a string, return the value. / 如果键存在且值为字符串，返回值。
    /// - If the key exists but has a wrong type, return an error. / 如果键存在但类型错误，返回错误。
    /// - If the key does not exist, return a null response. / 如果键不存在，返回 null 响应。
    pub fn get_command(
        db: &mut Arc<Mutex<Db>>,
        parse: &mut Parse,
    ) -> crate::Result<Frame> {
        match Self::parse_command(parse) {
            Ok(get) => match db.lock().unwrap().get(&get.key) {
                // If the key exists and its value is a string, return the value
                // 如果键存在且值为字符串，返回值
                Some(DbType::String(s)) => {
                    Ok(Frame::Bulk(s.clone().into_bytes()))  // Return the value as a Bulk Frame / 将值作为 Bulk Frame 返回
                }
                // If the key exists but has a wrong type, return an error
                // 如果键存在但类型错误，返回错误
                Some(_) => Ok(Frame::Error("WRONGTYPE Operation against a key holding the wrong kind of value".to_string())),
                // If the key doesn't exist, return null
                // 如果键不存在，返回 Null
                None => Ok(Frame::Null),
            },
            // If command parsing fails, return an error
            // 如果命令解析失败，返回错误
            Err(_) => {
                Ok(Frame::Error("ERR wrong number of arguments for 'get' command".to_string()))
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
    /// Returns the parsed `Get` instance containing the key. / 返回解析后的 `Get` 实例，包含要获取值的键。
    fn parse_command(parse: &mut Parse) -> crate::Result<Self> {
        let key = parse.next_string()?;  // Get the key to be retrieved / 获取要检索的键

        Ok(Get { key })
    }
}
