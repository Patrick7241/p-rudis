use std::sync::{Arc, Mutex};
use crate::db::{Db, DbType};
use crate::frame::Frame;
use crate::parse::Parse;

/// `Msetnx` command for string type.
/// `Msetnx` 命令用于字符串类型。
///
/// Sets multiple keys with respective values. The operation is performed only if none of the keys already exist.
/// 如果所有的键都不存在，则设置它们的值；如果有任何一个键已经存在，则不执行操作，返回 `0`。
/// If all keys do not exist, the key-value pairs are set and `1` is returned to indicate success.
/// 如果所有键都不存在，则设置键值对并返回 `1`，表示操作成功。
/// Returns `0` if at least one key already exists, and no operation is performed.
/// 如果至少有一个键已经存在，则返回 `0`，表示操作未执行。
pub struct Msetnx {
    keys_values: Vec<(String, String)>,  // List of key-value pairs to set / 键值对列表，用于设置
}

impl Msetnx {
    /// Executes the `msetnx` command.
    /// 执行 `msetnx` 命令。
    ///
    /// # Arguments
    /// # 参数
    /// - `db`: Shared reference to the database for access. / 用于访问数据库的共享引用。
    /// - `parse`: For parsing the command from the client. / 用于解析客户端传来的命令。
    ///
    /// # Return
    /// # 返回
    /// - Returns `1` if the keys were successfully set. / 如果键值成功设置，返回 `1`。
    /// - Returns `0` if any of the keys already exist. / 如果任何一个键已经存在，返回 `0`。
    pub fn msetnx_command(
        db: &mut Arc<Mutex<Db>>,
        parse: &mut Parse,
    ) -> crate::Result<Frame> {
        match Msetnx::parse_command(parse) {
            Ok(msetnx) => {
                let mut db = db.lock().unwrap();

                // Check if any of the keys already exist
                // 检查所有键是否存在
                for (key, _) in &msetnx.keys_values {
                    if db.exists(key) {
                        // If any key exists, return 0 (no operation)
                        // 如果有任何一个键已经存在，返回 0
                        return Ok(Frame::Integer(0));
                    }
                }

                // If all keys do not exist, set them
                // 如果所有键都不存在，则设置它们
                for (key, value) in msetnx.keys_values {
                    db.set(&key, DbType::String(value), None);
                }

                // Return 1 to indicate success
                // 设置成功，返回 1
                Ok(Frame::Integer(1))
            }
            // If the command has incorrect arguments, return an error
            // 如果命令参数不正确，返回错误
            Err(_) => {
                Ok(Frame::Error("ERR wrong number of arguments for 'msetnx' command".to_string()))
            }
        }
    }

    /// Parses the command and retrieves the key-value pairs.
    /// 解析命令并获取键值对。
    ///
    /// # Arguments
    /// # 参数
    /// - `parse`: The `Parse` instance used to parse the command. / 用于解析命令的 `Parse` 实例。
    ///
    /// # Return
    /// # 返回
    /// Returns the parsed `Msetnx` instance containing the key-value pairs. / 返回解析后的 `Msetnx` 实例，包含键值对。
    fn parse_command(parse: &mut Parse) -> crate::Result<Self> {
        let mut keys_values = Vec::new();

        // Parse the key-value pairs from the command
        // 解析命令中的键值对
        while let Ok(key) = parse.next_string() {
            let value = parse.next_string()?;
            keys_values.push((key, value));
        }

        // If the number of key-value pairs is odd or empty, return an error
        // 如果键值对的数量为奇数或为空，返回错误
        if keys_values.is_empty() || keys_values.len() % 2 != 0 {
            return Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "ERR wrong number of arguments for 'msetnx' command",
            )));
        }

        Ok(Msetnx { keys_values })
    }
}
