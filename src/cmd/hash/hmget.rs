use std::sync::Arc;
use std::sync::Mutex;
use crate::db::{Db, DbType};
use crate::frame::Frame;
use crate::parse::Parse;

/// Represents the `HMGET` command in a Redis-like system.
///
/// The `HMGET` command retrieves the values of multiple fields in the hash stored at the specified key.
/// If a field does not exist, it returns `nil` for that field. The return value is an array of `Bulk` values,
/// where each field's value is returned as `Bulk`. If a field does not exist, it returns `Null` for that field.
///
/// 表示 Redis 风格系统中的 `HMGET` 命令。
///
/// `HMGET` 命令获取存储在指定键的哈希表中的多个字段的值。
/// 如果字段不存在，则返回该字段的 `nil`。返回值是一个包含 `Bulk` 数据类型的数组，
/// 每个字段的值以 `Bulk` 返回。如果字段不存在，则该字段返回 `Null`。
pub struct Hmget {
    key: String,          // The key of the hash in the database. / 数据库中哈希表的键。
    fields: Vec<String>,  // The list of fields to retrieve. / 要检索的字段列表。
}

impl Hmget {
    /// Executes the `HMGET` command.
    ///
    /// This function processes the parsed command and retrieves the values of the specified fields from the hash
    /// stored at the given key. It handles the following scenarios:
    ///
    /// - If the key exists and contains a hash, it returns the values of the specified fields.
    /// - If the key exists but is not a hash, it returns a `WRONGTYPE` error.
    /// - If the key does not exist, it returns `nil` for each requested field.
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
    /// Returns an `Array` frame representing the values of the requested fields. If the fields do not exist, it returns `Null`.
    ///
    /// 返回一个 `Array` 类型的帧，表示请求的字段的值。如果字段不存在，则返回 `Null`。
    pub fn hmget_command(
        db: &mut Arc<Mutex<Db>>,
        parse: &mut Parse
    ) -> crate::Result<Frame> {
        match Hmget::parse_command(parse) {
            Ok(hmget) => {
                let mut db = db.lock().unwrap();
                match db.get(&hmget.key) {
                    Some(DbType::Hash(hash)) => {
                        // Get the values for each field. / 为每个字段获取值
                        let mut result = Vec::new();
                        for field in hmget.fields {
                            if let Some(value) = hash.get(&field) {
                                // If the field exists, return its value. / 如果字段存在，返回字段的值
                                result.push(Frame::Bulk(value.clone().into_bytes()));
                            } else {
                                // If the field does not exist, return nil. / 如果字段不存在，返回 nil
                                result.push(Frame::Null);
                            }
                        }

                        // Return the list of field values. / 返回字段值的列表
                        Ok(Frame::Array(result))
                    },
                    Some(_) => {
                        // Key exists but type mismatch, return WRONGTYPE error. / 键存在，但类型不匹配，返回 WRONGTYPE 错误。
                        Ok(Frame::Error("WRONGTYPE Operation against a key holding the wrong kind of value".to_string()))
                    },
                    None => {
                        // Key does not exist, return nil for each requested field. / 键不存在，为每个请求的字段返回 nil
                        let result = vec![Frame::Null; hmget.fields.len()];
                        Ok(Frame::Array(result))
                    }
                }
            }
            Err(_) => {
                // Incorrect number of arguments, return error. / 参数数量错误，返回错误。
                Ok(Frame::Error("ERR wrong number of arguments for 'hmget' command".to_string()))
            }
        }
    }

    /// Parses the command and retrieves the key and fields for the hash.
    ///
    /// 解析命令并获取哈希表的键和字段。
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing the `Hmget` struct with the parsed key and fields if successful.
    /// Otherwise, returns an error frame indicating the problem.
    ///
    /// 如果成功，返回包含解析后的键和字段的 `Hmget` 结构体。如果失败，返回错误帧以指示问题。
    fn parse_command(parse: &mut Parse) -> crate::Result<Self> {
        // The command requires at least two arguments: the key and at least one field.
        // 命令需要至少两个参数：键和至少一个字段。
        if parse.args_number()? < 2 {
            return Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, "ERR wrong number of arguments for 'hmget' command")));
        }

        let key = parse.next_string()?;
        let mut fields = Vec::new();

        // Parse all fields. / 解析所有字段
        while let Ok(field) = parse.next_string() {
            fields.push(field);
        }

        Ok(Hmget { key, fields })
    }
}
