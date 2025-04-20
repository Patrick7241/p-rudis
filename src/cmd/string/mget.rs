use std::sync::Arc;
use std::sync::Mutex;
use crate::db::{Db, DbType};
use crate::frame::Frame;
use crate::parse::Parse;

/// `Mget` command for string type.
/// `Mget` 命令用于字符串类型。
///
/// Gets the values of multiple specified keys. If a key exists, returns its value. If a key does not exist, returns `nil`.
/// 获取多个指定键的字符串值。如果键存在，则返回该键的值，如果不存在，则返回 `nil`。
/// Returns a list containing all the values of the specified keys (if a key does not exist, returns `nil`).
/// 返回一个包含所有指定键的值的列表（如果某个键不存在，则为 `nil`）。
pub struct Mget {
    keys: Vec<String>,  // The list of keys to retrieve / 要获取的键的列表
}

impl Mget {
    /// Executes the `mget` command.
    /// 执行 `mget` 命令。
    ///
    /// # Arguments
    /// # 参数
    /// - `db`: Shared reference to the database for access. / 用于访问数据库的共享引用。
    /// - `parse`: For parsing the command from the client. / 用于解析客户端传来的命令。
    ///
    /// # Return
    /// # 返回
    /// - Returns an array with the values of the specified keys. If a key does not exist, returns `null` for that key. / 返回一个数组，包含指定键的值。如果键不存在，则该键的值为 `null`。
    pub fn mget_command(
        db: &mut Arc<Mutex<Db>>,
        parse: &mut Parse
    ) -> crate::Result<Frame> {
        match Mget::parse_command(parse) {
            Ok(mget) => {
                let mut db = db.lock().unwrap();

                let mut result = Vec::new();
                for key in mget.keys {
                    // Get the value for each key
                    // 获取每个键的值
                    match db.get(&key) {
                        // If the key exists and its value is a string, return its value
                        // 如果键存在且值为字符串，返回其值
                        Some(DbType::String(value)) => {
                            result.push(Frame::Simple(value.to_string()));
                        }
                        // If the key exists but its value is not a string, return an error
                        // 如果键存在但值不是字符串，返回错误
                        Some(_) => {
                            result.push(Frame::Error("WRONGTYPE Operation against a key holding the wrong kind of value".to_string()))
                        }
                        // If the key does not exist, return null
                        // 如果键不存在，返回 null
                        None => {
                            result.push(Frame::Null); // The key does not exist or is not a string
                        }
                    }
                }

                // Return the list of values
                // 返回包含所有值的列表
                Ok(Frame::Array(result))
            }
            // If the command has an incorrect number of arguments, return an error
            // 如果命令参数个数不正确，返回错误
            Err(_) => {
                Ok(Frame::Error("ERR wrong number of arguments for 'mget' command".to_string()))
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
    /// Returns the parsed `Mget` instance containing the list of keys. / 返回解析后的 `Mget` 实例，包含键的列表。
    fn parse_command(parse: &mut Parse) -> crate::Result<Self> {
        let mut keys = Vec::new();

        // Parse all keys in the command
        // 解析命令中的所有键
        while let Ok(key) = parse.next_string() {
            keys.push(key);
        }

        // If no keys are provided, return an error
        // 如果没有提供任何键，返回错误
        if keys.is_empty() {
            return Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, "ERR wrong number of arguments for 'mget' command")));
        }

        Ok(Mget { keys })
    }
}
