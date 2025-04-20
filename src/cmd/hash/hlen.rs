use std::sync::Arc;
use std::sync::Mutex;
use crate::db::{Db, DbType};
use crate::frame::Frame;
use crate::parse::Parse;

/// Represents the `HLEN` command in a Redis-like system.
///
/// The `HLEN` command retrieves the number of fields in the hash stored at the specified key.
/// If the hash does not exist, it returns `nil`. The return value is an integer representing the number of fields in the hash.
///
/// 表示 Redis 风格系统中的 `HLEN` 命令。
///
/// `HLEN` 命令获取存储在指定键的哈希表中的字段数量。
/// 如果哈希表不存在，返回 `nil`。返回值为一个整数，表示哈希表中的字段数量。
pub struct Hlen {
    key: String,  // The key of the hash in the database. / 数据库中哈希表的键。
}

impl Hlen {
    /// Executes the `HLEN` command.
    ///
    /// This function processes the parsed command and retrieves the number of fields from the hash
    /// stored at the given key. It handles the following scenarios:
    ///
    /// - If the key exists and contains a hash, it returns the number of fields in the hash.
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
    /// Returns an `Integer` frame representing the number of fields in the hash. If the hash does not exist, returns `0`.
    ///
    /// 返回一个 `Integer` 类型的帧，表示哈希表中的字段数量。如果哈希表不存在，返回 `0`。
    pub fn hlen_command(
        db: &mut Arc<Mutex<Db>>,
        parse: &mut Parse
    ) -> crate::Result<Frame> {
        match Hlen::parse_command(parse) {
            Ok(hlen) => {
                let mut db = db.lock().unwrap();
                match db.get(&hlen.key) {
                    Some(DbType::Hash(hash)) => {
                        // Get the number of fields in the hash. / 获取哈希表中的字段数量
                        let field_count = hash.len();
                        // Return the field count. / 返回字段数量
                        Ok(Frame::Integer(field_count as i64))
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
                Ok(Frame::Error("ERR wrong number of arguments for 'hlen' command".to_string()))
            }
        }
    }

    /// Parses the command and retrieves the key for the hash.
    ///
    /// 解析命令并获取哈希表的键。
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing the `Hlen` struct with the parsed key if successful.
    /// Otherwise, returns an error frame indicating the problem.
    ///
    /// 如果成功，返回包含解析后的键的 `Hlen` 结构体。如果失败，返回错误帧以指示问题。
    fn parse_command(parse: &mut Parse) -> crate::Result<Self> {
        // The command requires exactly one argument: the key.
        // 命令需要恰好一个参数：键。
        if parse.args_number()? != 1 {
            return Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, "ERR wrong number of arguments for 'hlen' command")));
        }

        let key = parse.next_string()?;

        Ok(Hlen { key })
    }
}
