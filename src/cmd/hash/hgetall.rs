use std::sync::Arc;
use std::sync::Mutex;
use crate::db::{Db, DbType};
use crate::frame::Frame;
use crate::parse::Parse;

/// Represents the `HGETALL` command in a Redis-like system.
///
/// The `HGETALL` command retrieves all the fields and values in the hash stored at the specified key.
/// It returns an array of fields and values. If the hash does not exist, it returns an empty array.
///
/// 表示 Redis 风格系统中的 `HGETALL` 命令。
///
/// `HGETALL` 命令获取存储在指定键的哈希表中的所有字段和值。
/// 它返回一个包含字段和值的数组。如果哈希表不存在，返回空数组。
pub struct Hgetall {
    key: String,  // The key of the hash in the database. / 数据库中哈希表的键。
}

impl Hgetall {
    /// Executes the `HGETALL` command.
    ///
    /// This function processes the parsed command and retrieves all the fields and values from the hash
    /// stored at the given key. It handles the following scenarios:
    ///
    /// - If the key exists and contains a hash, it returns all the fields and their corresponding values.
    /// - If the key exists but is not a hash, it returns a `WRONGTYPE` error.
    /// - If the key does not exist, it returns an empty list message.
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
    /// Returns an array of `Bulk` frames, where each field and its value are stored as `Bulk`.
    /// If the hash does not exist, it returns an empty list message.
    ///
    /// 返回一个包含字段和值的 `Bulk` 类型帧数组。如果哈希表不存在，返回空数组信息。
    pub fn hgetall_command(
        db: &mut Arc<Mutex<Db>>,
        parse: &mut Parse
    ) -> crate::Result<Frame> {
        match Hgetall::parse_command(parse) {
            Ok(hgetall) => {
                let mut db = db.lock().unwrap();
                match db.get(&hgetall.key) {
                    Some(DbType::Hash(hash)) => {
                        let mut result = Vec::new();

                        // Iterate over each field and value in the hash. / 遍历哈希表中的每个字段和值
                        for (field, value) in hash.iter() {
                            // Add field and value to the result. / 将字段和值添加到结果中
                            result.push(Frame::Bulk(field.clone().into_bytes()));
                            result.push(Frame::Bulk(value.clone().into_bytes()));
                        }

                        // Return the result, multiple Bulk data types. / 返回结果，多个 Bulk 数据类型
                        Ok(Frame::Array(result))
                    },
                    Some(_) => {
                        // Key exists, but type mismatch, return WRONGTYPE error. / 键存在，但类型不匹配，返回 WRONGTYPE 错误。
                        Ok(Frame::Error("WRONGTYPE Operation against a key holding the wrong kind of value".to_string()))
                    },
                    None => {
                        // Key does not exist, return an empty list message. / 键不存在，返回空列表信息
                        Ok(Frame::Simple("(empty list or set)".to_string()))
                    }
                }
            }
            Err(_) => {
                // Incorrect number of arguments, return error. / 参数数量错误，返回错误。
                Ok(Frame::Error("ERR wrong number of arguments for 'hgetall' command".to_string()))
            }
        }
    }

    /// Parses the command and retrieves the key for the hash.
    ///
    /// 解析命令并获取哈希表的键。
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing the `Hgetall` struct with the parsed key if successful.
    /// Otherwise, returns an error frame indicating the problem.
    ///
    /// 如果成功，返回包含解析后的键的 `Hgetall` 结构体。如果失败，返回错误帧以指示问题。
    fn parse_command(parse: &mut Parse) -> crate::Result<Self> {
        // The command requires exactly one argument: the key.
        // 命令需要恰好一个参数：键。
        if parse.args_number()? != 1 {
            return Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, "ERR wrong number of arguments for 'hgetall' command")));
        }
        let key = parse.next_string()?;

        Ok(Hgetall { key })
    }
}
