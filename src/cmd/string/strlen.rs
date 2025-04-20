use std::sync::Arc;
use std::sync::Mutex;
use crate::db::{Db, DbType};
use crate::frame::Frame;
use crate::parse::Parse;

/// `Strlen` command for string type.
/// 获取指定键的字符串值的长度。如果键不存在，返回 0。
///
/// The `strlen` command returns the length of the string value of a given key.
/// If the key does not exist or the key is not a string, it returns 0.
pub struct Strlen {
    key: String,  // The key to retrieve the length of its value / 要获取其值长度的键
}

impl Strlen {
    /// Executes the `strlen` command.
    /// 执行 `strlen` 命令。
    ///
    /// # Arguments
    /// - `db`: Shared reference to the database for access. / 用于访问数据库的共享引用。
    /// - `parse`: For parsing the command from the client. / 用于解析客户端传来的命令。
    ///
    /// # Return
    /// - Returns the length of the string if the key exists and is a string. / 如果键存在且是字符串，则返回字符串的长度。
    /// - Returns `0` if the key does not exist or is not a string. / 如果键不存在或不是字符串，则返回 0。
    pub fn strlen_command(
        db: &mut Arc<Mutex<Db>>,
        parse: &mut Parse
    ) -> crate::Result<Frame> {
        match Strlen::parse_command(parse) {
            Ok(strlen) => {
                let mut db = db.lock().unwrap();
                // Get the current value of the key
                // 获取键的当前值
                match db.get(&strlen.key) {
                    // If the key exists and is of type string, return its length
                    // 如果键存在并且是字符串类型，返回其长度
                    Some(DbType::String(value)) => Ok(Frame::Integer(value.len() as i64)),
                    // If the key does not exist or is not a string, return 0
                    // 如果键不存在或不是字符串类型，返回 0
                    _ => Ok(Frame::Integer(0)),
                }
            }
            // If there is an error in parsing, return an error frame
            // 如果解析时发生错误，返回错误响应
            Err(_) => {
                Ok(Frame::Error("ERR wrong number of arguments for 'strlen' command".to_string()))
            }
        }
    }

    /// Parses the command and retrieves the key.
    /// 解析命令并获取键。
    ///
    /// # Arguments
    /// - `parse`: The `Parse` instance used to parse the command. / 用于解析命令的 `Parse` 实例。
    ///
    /// # Return
    /// - Returns the parsed `Strlen` instance with the key. / 返回包含键的 `Strlen` 实例。
    fn parse_command(parse: &mut Parse) -> crate::Result<Self> {
        let key = parse.next_string()?;  // Parse the key / 解析键
        Ok(Strlen { key })  // Return the `Strlen` instance with the key / 返回包含键的 `Strlen` 实例
    }
}
