use std::sync::Arc;
use std::sync::Mutex;
use crate::db::{Db, DbType};
use crate::frame::Frame;
use crate::parse::Parse;
use std::collections::HashMap;

/// Represents the `HMSET` command in a Redis-like system.
///
/// The `HMSET` command sets multiple fields in the hash stored at the specified key.
/// If a field already exists, it updates its value. The command returns `OK` if successful.
///
/// 表示 Redis 风格系统中的 `HMSET` 命令。
///
/// `HMSET` 命令在指定键的哈希表中设置多个字段的值。
/// 如果字段已存在，则更新它的值。命令执行成功后返回 `OK`。
pub struct Hmset {
    key: String,                  // The key of the hash in the database. / 数据库中哈希表的键。
    fields_values: Vec<(String, String)>,  // A vector of (field, value) pairs to set in the hash. / 要在哈希表中设置的 (字段, 值) 配对的向量。
}

impl Hmset {
    /// Executes the `HMSET` command.
    ///
    /// This function processes the parsed command and sets the specified fields in the hash stored at the given key.
    /// It handles the following scenarios:
    ///
    /// - If the key exists and contains a hash, it updates or inserts the specified fields and values.
    /// - If the key exists but is not a hash, it returns a `WRONGTYPE` error.
    /// - If the key does not exist, it creates a new hash and sets the specified fields.
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
    /// Returns a `Simple` frame with the value `OK` indicating the success of the command.
    ///
    /// 返回一个 `Simple` 类型的帧，包含值 `OK`，表示命令执行成功。
    pub fn hmset_command(
        db: &mut Arc<Mutex<Db>>,
        parse: &mut Parse
    ) -> crate::Result<Frame> {
        match Hmset::parse_command(parse) {
            Ok(hmset) => {
                let mut db = db.lock().unwrap();
                match db.get_dbtype_mut(&hmset.key) {
                    Some(DbType::Hash(hash)) => {
                        // Iterate over the fields and values and insert or update them in the hash.
                        // 遍历字段和值，并插入或更新哈希表中的字段
                        for (field, value) in hmset.fields_values {
                            // If the field exists, update its value; if not, insert a new field.
                            // 如果字段存在，更新其值；如果不存在，插入新字段。
                            hash.insert(field, value);
                        }
                        Ok(Frame::Simple("OK".to_string()))  // Return "OK" indicating success.
                    },
                    Some(_) => {
                        // Key exists, but type mismatch; return WRONGTYPE error.
                        // 键存在，但类型不匹配，返回 WRONGTYPE 错误。
                        Ok(Frame::Error("WRONGTYPE Operation against a key holding the wrong kind of value".to_string()))
                    },
                    None => {
                        // Key does not exist; create a new hash and set the fields.
                        // 键不存在，创建一个新的哈希表，并设置字段。
                        let mut new_hash = HashMap::new();
                        for (field, value) in hmset.fields_values {
                            new_hash.insert(field, value);
                        }
                        db.set(&hmset.key, DbType::Hash(new_hash), None); // Set the new hash in the database.
                        Ok(Frame::Simple("OK".to_string()))  // Return "OK" indicating success.
                    }
                }
            }
            Err(_) => {
                // Incorrect number of arguments, return error. / 参数数量错误，返回错误。
                Ok(Frame::Error("ERR wrong number of arguments for 'hmset' command".to_string()))
            }
        }
    }

    /// Parses the command and retrieves the key and fields/values for the hash.
    ///
    /// 解析命令并获取哈希表的键和字段/值。
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing the `Hmset` struct with the parsed key and fields/values if successful.
    /// Otherwise, returns an error frame indicating the problem.
    ///
    /// 如果成功，返回包含解析后的键和字段/值的 `Hmset` 结构体。如果失败，返回错误帧以指示问题。
    fn parse_command(parse: &mut Parse) -> crate::Result<Self> {
        // The command requires at least three arguments: the key and at least one field-value pair.
        // 命令需要至少三个参数：键和至少一个字段-值对。
        if parse.args_number()? < 3 || parse.args_number()? % 2 != 1 {
            return Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, "ERR wrong number of arguments for 'hmset' command")));
        }

        let key = parse.next_string()?;
        let mut fields_values = Vec::new();

        // Parse the field-value pairs. / 解析字段-值对
        while let Ok(field) = parse.next_string() {
            let value = parse.next_string()?;
            fields_values.push((field, value));
        }

        Ok(Hmset {
            key,
            fields_values,
        })
    }
}
