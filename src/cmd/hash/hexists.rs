use std::sync::Arc;
use std::sync::Mutex;
use crate::db::{Db, DbType};
use crate::frame::Frame;
use crate::parse::Parse;

/// Represents the `HEXISTS` command in a Redis-like system.
///
/// The `HEXISTS` command checks if a field exists in the hash stored at the specified key.
/// It returns `1` if the field exists, and `0` if the field does not exist. The return value is an `Integer` type.
///
/// 表示 Redis 风格系统中的 `HEXISTS` 命令。
///
/// `HEXISTS` 命令用于检查指定字段是否存在于存储在给定键的哈希表中。
/// 如果字段存在，返回 `1`，如果字段不存在，返回 `0`。返回值为 `Integer` 类型。
pub struct Hexists {
    key: String,               // The key of the hash in the database. / 数据库中哈希表的键。
    field: String,             // The field to check for existence in the hash. / 要检查是否存在的字段。
}

impl Hexists {
    /// Executes the `HEXISTS` command.
    ///
    /// This function processes the parsed command and checks if the specified field exists in the hash
    /// stored at the given key. It handles the following scenarios:
    ///
    /// - If the key exists and contains a hash, it checks for the field's existence and returns `1` or `0`.
    /// - If the key exists but is not a hash, it returns a `WRONGTYPE` error.
    /// - If the key does not exist, it returns `0`.
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
    /// Returns `1` if the field exists, `0` if it does not exist, or an error frame if the operation fails.
    ///
    /// 返回 `1` 如果字段存在，`0` 如果字段不存在，或者如果操作失败返回错误帧。
    pub fn hexists_command(
        db: &mut Arc<Mutex<Db>>,
        parse: &mut Parse
    ) -> crate::Result<Frame> {
        match Hexists::parse_command(parse) {
            Ok(hexists) => {
                let mut db = db.lock().unwrap();
                match db.get(&hexists.key) {
                    Some(DbType::Hash(hash)) => {
                        // Check if the field exists in the hash. / 检查字段是否存在于哈希表中。
                        if hash.contains_key(&hexists.field) {
                            Ok(Frame::Integer(1)) // Field exists, return 1. / 字段存在，返回 1。
                        } else {
                            Ok(Frame::Integer(0)) // Field does not exist, return 0. / 字段不存在，返回 0。
                        }
                    },
                    Some(_) => {
                        // Key exists but type mismatch, return WRONGTYPE error. / 键存在，但类型不匹配，返回 WRONGTYPE 错误。
                        Ok(Frame::Error("WRONGTYPE Operation against a key holding the wrong kind of value".to_string()))
                    },
                    None => {
                        // Key does not exist, return 0. / 键不存在，返回 0。
                        Ok(Frame::Integer(0))
                    }
                }
            }
            Err(_) => {
                // Incorrect number of arguments, return error. / 参数数量错误，返回错误。
                Ok(Frame::Error("ERR wrong number of arguments for 'hexists' command".to_string()))
            }
        }
    }

    /// Parses the command and extracts the key and field.
    ///
    /// 解析命令并提取键和字段。
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing the `Hexists` struct with the parsed key and field if successful.
    /// Otherwise, returns an error frame indicating the problem.
    ///
    /// 如果成功，返回包含解析后的键和字段的 `Hexists` 结构体。如果失败，返回错误帧以指示问题。
    fn parse_command(parse: &mut Parse) -> crate::Result<Self> {
        // The command requires exactly two arguments: key and field.
        // 命令需要恰好两个参数：键和字段。
        if parse.args_number()? != 2 {
            return Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, "ERR wrong number of arguments for 'hexists' command")));
        }

        let key = parse.next_string()?;
        let field = parse.next_string()?;

        Ok(Hexists { key, field })
    }
}
