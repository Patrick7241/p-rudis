use std::sync::Arc;
use std::sync::Mutex;
use crate::db::{Db, DbType};
use crate::frame::Frame;
use crate::parse::Parse;

/// Represents the `HKEYS` command in a Redis-like system.
///
/// The `HKEYS` command retrieves all the field names in the hash stored at the specified key.
/// If the hash does not exist, it returns `nil`. The return value is an `Array` containing all the field names in the hash.
///
/// 表示 Redis 风格系统中的 `HKEYS` 命令。
///
/// `HKEYS` 命令获取存储在指定键的哈希表中的所有字段名。
/// 如果哈希表不存在，返回 `nil`。返回值为一个 `Array`，包含哈希表中的所有字段名。
pub struct Hkeys {
    key: String,  // The key of the hash in the database. / 数据库中哈希表的键。
}

impl Hkeys {
    /// Executes the `HKEYS` command.
    ///
    /// This function processes the parsed command and retrieves all the field names from the hash
    /// stored at the given key. It handles the following scenarios:
    ///
    /// - If the key exists and contains a hash, it returns all the field names.
    /// - If the key exists but is not a hash, it returns a `WRONGTYPE` error.
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
    /// Returns an array of `Bulk` frames, where each field name is stored as `Bulk`.
    /// If the hash does not exist, it returns `nil`.
    ///
    /// 返回一个包含字段名的 `Bulk` 类型帧数组。如果哈希表不存在，返回 `nil`。
    pub fn hkeys_command(
        db: &mut Arc<Mutex<Db>>,
        parse: &mut Parse
    ) -> crate::Result<Frame> {
        match Hkeys::parse_command(parse) {
            Ok(hkeys) => {
                let mut db = db.lock().unwrap();
                match db.get(&hkeys.key) {
                    Some(DbType::Hash(hash)) => {
                        // Get all field names. / 获取所有的字段名
                        let fields: Vec<Frame> = hash.keys()
                            .map(|field| Frame::Bulk(field.clone().into_bytes())) // Convert field names to Frame::Bulk / 将字段名转化为 Frame::Bulk
                            .collect();

                        // Return the array of field names. / 返回字段名的数组
                        Ok(Frame::Array(fields))
                    },
                    Some(_) => {
                        // Key exists but type mismatch, return WRONGTYPE error. / 键存在，但类型不匹配，返回 WRONGTYPE 错误。
                        Ok(Frame::Error("WRONGTYPE Operation against a key holding the wrong kind of value".to_string()))
                    },
                    None => {
                        // Key does not exist, return nil. / 键不存在，返回 nil。
                        Ok(Frame::Null)
                    }
                }
            }
            Err(_) => {
                // Incorrect number of arguments, return error. / 参数数量错误，返回错误。
                Ok(Frame::Error("ERR wrong number of arguments for 'hkeys' command".to_string()))
            }
        }
    }

    /// Parses the command and retrieves the key for the hash.
    ///
    /// 解析命令并获取哈希表的键。
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing the `Hkeys` struct with the parsed key if successful.
    /// Otherwise, returns an error frame indicating the problem.
    ///
    /// 如果成功，返回包含解析后的键的 `Hkeys` 结构体。如果失败，返回错误帧以指示问题。
    fn parse_command(parse: &mut Parse) -> crate::Result<Self> {
        // The command requires exactly one argument: the key.
        // 命令需要恰好一个参数：键。
        if parse.args_number()? != 1 {
            return Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, "ERR wrong number of arguments for 'hkeys' command")));
        }

        let key = parse.next_string()?;

        Ok(Hkeys { key })
    }
}
