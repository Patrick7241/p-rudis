use std::sync::Arc;
use std::sync::Mutex;
use crate::db::{Db, DbType};
use crate::frame::Frame;
use crate::parse::Parse;

/// Represents the `HVALS` command in a Redis-like system.
///
/// The `HVALS` command retrieves all the values in a hash stored at the specified key.
/// It returns an array of all values in the hash, or `nil` if the key does not exist.
///
/// 返回哈希表中所有字段的值。如果哈希表不存在，则返回 `nil`。
///
/// `HVALS` 命令获取指定键下哈希表的所有字段值。返回包含哈希表中所有字段值的数组，或者如果键不存在，返回 `nil`。
pub struct Hvals {
    key: String,   // The key of the hash in the database. / 数据库中哈希表的键
}

impl Hvals {
    /// Executes the `HVALS` command.
    ///
    /// This function processes the parsed command and retrieves all the values of the fields in the hash stored at the specified key.
    /// It handles the following scenarios:
    ///
    /// - If the key exists and contains a hash, it retrieves all the values of the fields in the hash.
    /// - If the key does not exist, it returns `nil`.
    ///
    /// # Arguments
    ///
    /// - `db`: A mutable reference to the database (`Arc<Mutex<Db>>`), where the hash is stored.
    ///         / 数据库 (`Arc<Mutex<Db>>`) 的可变引用，存储哈希表的位置。
    /// - `parse`: A reference to the parser that contains the parsed command.
    ///            / 解析器的引用，包含解析后的命令。
    ///
    /// # Returns
    ///
    /// Returns an `Array` frame containing the values of all fields in the hash if the key exists, or `Null` if the key does not exist.
    ///
    /// 返回一个 `Array` 类型的帧，包含哈希表中所有字段的值，如果键存在；如果键不存在，则返回 `Null`。
    pub fn hvals_command(
        db: &mut Arc<Mutex<Db>>,
        parse: &mut Parse
    ) -> crate::Result<Frame> {
        match Hvals::parse_command(parse) {
            Ok(hvals) => {
                let mut db = db.lock().unwrap();
                match db.get(&hvals.key) {
                    Some(DbType::Hash(hash)) => {
                        // Get all field values from the hash
                        // 获取哈希表中所有字段的值
                        let values: Vec<Frame> = hash.values()
                            .map(|value| Frame::Bulk(value.clone().into_bytes())) // Convert field values to Frame::Bulk
                            .collect();

                        // Return the array of field values
                        // 返回包含字段值的数组
                        Ok(Frame::Array(values))
                    },
                    Some(_) => {
                        // If the key exists but is not a hash, return WRONGTYPE error
                        // 如果键存在但不是哈希表，返回 WRONGTYPE 错误
                        Ok(Frame::Error("WRONGTYPE Operation against a key holding the wrong kind of value".to_string()))
                    },
                    None => {
                        // If the key does not exist, return nil
                        // 如果键不存在，返回 nil
                        Ok(Frame::Null)
                    }
                }
            }
            Err(_) => {
                // If the number of arguments is wrong, return an error
                // 如果参数数量错误，返回错误
                Ok(Frame::Error("ERR wrong number of arguments for 'hvals' command".to_string()))
            }
        }
    }

    /// Parses the command, ensuring there is exactly 1 argument: the key.
    ///
    /// 解析命令，确保参数为 1 个：键。
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing the `Hvals` struct with the parsed key if successful.
    /// Otherwise, returns an error frame indicating the problem.
    ///
    /// 如果成功，返回包含解析后的键的 `Hvals` 结构体。如果失败，返回错误帧以指示问题。
    fn parse_command(parse: &mut Parse) -> crate::Result<Self> {
        if parse.args_number()? != 1 {
            return Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, "ERR wrong number of arguments for 'hvals' command")));
        }

        let key = parse.next_string()?;

        Ok(Hvals { key })
    }
}
