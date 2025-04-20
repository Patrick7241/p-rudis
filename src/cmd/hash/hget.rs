use std::sync::Arc;
use std::sync::Mutex;
use crate::db::{Db, DbType};
use crate::frame::Frame;
use crate::parse::Parse;

/// Represents the `HGET` command in a Redis-like system.
///
/// The `HGET` command retrieves the value of a specified field from a hash stored at the specified key.
/// If the field does not exist, it returns `nil`. The return value is a `Bulk` type containing the value of the field.
///
/// 表示 Redis 风格系统中的 `HGET` 命令。
///
/// `HGET` 命令从指定键存储的哈希表中获取指定字段的值。
/// 如果字段不存在，返回 `nil`。返回值为 `Bulk` 类型，包含字段的值。
pub struct Hget {
    key: String,               // The key of the hash in the database. / 数据库中哈希表的键。
    field: String,             // The field whose value is to be retrieved from the hash. / 要从哈希表中获取值的字段。
}

impl Hget {
    /// Executes the `HGET` command.
    ///
    /// This function processes the parsed command and attempts to retrieve the value of the specified field from the hash
    /// stored at the given key. It handles the following scenarios:
    ///
    /// - If the key exists and contains a hash, it retrieves the value of the specified field.
    /// - If the field does not exist, it returns `nil`.
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
    /// Returns the value of the specified field if it exists, or `nil` if the field does not exist.
    /// If the operation fails, returns an error frame.
    ///
    /// 如果字段存在，返回指定字段的值；如果字段不存在，返回 `nil`。
    /// 如果操作失败，返回错误帧。
    pub fn hget_command(
        db: &mut Arc<Mutex<Db>>,
        parse: &mut Parse
    ) -> crate::Result<Frame> {
        match Hget::parse_command(parse) {
            Ok(hget) => {
                let mut db = db.lock().unwrap();
                match db.get(&hget.key) {
                    Some(DbType::Hash(hash)) => {
                        // If the field exists, return its value. / 如果字段存在，则返回其值
                        if let Some(value) = hash.get(&hget.field) {
                            Ok(Frame::Bulk(value.clone().into_bytes())) // Return the value of the field. / 返回字段的值
                        } else {
                            // If the field does not exist, return nil. / 如果字段不存在，返回 nil。
                            Ok(Frame::Null)
                        }
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
                Ok(Frame::Error("ERR wrong number of arguments for 'hget' command".to_string()))
            }
        }
    }

    /// Parses the command and extracts the key and field.
    ///
    /// 解析命令并提取键和字段。
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing the `Hget` struct with the parsed key and field if successful.
    /// Otherwise, returns an error frame indicating the problem.
    ///
    /// 如果成功，返回包含解析后的键和字段的 `Hget` 结构体。如果失败，返回错误帧以指示问题。
    fn parse_command(parse: &mut Parse) -> crate::Result<Self> {
        // The command requires exactly two arguments: key and field.
        // 命令需要恰好两个参数：键和字段。
        if parse.args_number()? != 2 {
            return Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, "ERR wrong number of arguments for 'hget' command")));
        }

        let key = parse.next_string()?;
        let field = parse.next_string()?;

        Ok(Hget { key, field })
    }
}
