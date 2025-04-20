use std::sync::Arc;
use std::sync::Mutex;
use crate::db::{Db, DbType};
use crate::frame::Frame;
use crate::parse::Parse;

/// `Mset` command for string type.
/// `Mset` 命令用于字符串类型。
///
/// Sets multiple keys to their respective values. If a key already exists, its value is overwritten.
/// 设置多个键的值。如果某个键已经存在，则覆盖它的值。
/// Returns `OK` to indicate successful execution.
/// 返回 `OK` 表示命令执行成功。
pub struct Mset {
    keys_values: Vec<(String, String)>,  // The list of key-value pairs to set / 键值对列表，用于设置
}

impl Mset {
    /// Executes the `mset` command.
    /// 执行 `mset` 命令。
    ///
    /// # Arguments
    /// # 参数
    /// - `db`: Shared reference to the database for access. / 用于访问数据库的共享引用。
    /// - `parse`: For parsing the command from the client. / 用于解析客户端传来的命令。
    ///
    /// # Return
    /// # 返回
    /// - Returns `OK` to indicate success. / 返回 `OK` 表示成功。
    pub fn mset_command(
        db: &mut Arc<Mutex<Db>>,
        parse: &mut Parse
    ) -> crate::Result<Frame> {
        match Mset::parse_command(parse) {
            Ok(mset) => {
                let mut db = db.lock().unwrap();

                // Iterate over the key-value pairs and set each key's value
                // 遍历键值对，设置每个键的值
                for (key, value) in mset.keys_values {
                    db.set(&key, DbType::String(value), None);
                }

                // Return success response
                // 返回成功响应
                Ok(Frame::Simple("OK".to_string()))
            }
            // If the command has an incorrect number of arguments, return an error
            // 如果命令参数个数不正确，返回错误
            Err(_) => {
                Ok(Frame::Error("ERR wrong number of arguments for 'mset' command".to_string()))
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
    /// Returns the parsed `Mset` instance containing the key-value pairs. / 返回解析后的 `Mset` 实例，包含键值对。
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
            return Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, "ERR wrong number of arguments for 'mset' command")));
        }

        Ok(Mset { keys_values })
    }
}
