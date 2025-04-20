use std::collections::HashMap;
use std::sync::Arc;
use std::sync::Mutex;
use crate::db::{Db, DbType};
use crate::frame::Frame;
use crate::parse::Parse;

/// Represents the `HSET` command in a Redis-like system.
///
/// The `HSET` command sets the value of a field in a hash stored at the specified key.
/// If the field already exists, it updates the value of that field. The command returns:
/// - `1` if the field is newly added to the hash.
/// - `0` if the field already exists and is updated.
///
/// 表示 Redis 风格系统中的 `HSET` 命令。
///
/// `HSET` 命令在指定键的哈希表中设置字段的值。如果字段已存在，则更新该字段的值。命令返回：
/// - 如果字段是新添加的，返回 `1`。
/// - 如果字段已存在并更新，返回 `0`。
pub struct Hset {
    key: String,   // The key of the hash in the database. / 数据库中哈希表的键。
    field: String, // The field to set in the hash. / 要在哈希表中设置的字段。
    value: String, // The value to associate with the field. / 要与字段关联的值。
}

impl Hset {
    /// Executes the `HSET` command.
    ///
    /// This function processes the parsed command and sets the specified field in the hash stored at the given key.
    /// It handles the following scenarios:
    ///
    /// - If the key exists and contains a hash, it either adds or updates the specified field and value.
    /// - If the key does not exist, it creates a new hash and sets the field and value.
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
    /// Returns an `Integer` frame with either `1` (if the field was newly added) or `0` (if the field was updated).
    ///
    /// 返回一个 `Integer` 类型的帧，值为 `1`（如果字段是新添加的）或 `0`（如果字段已更新）。
    pub fn hset_command(
        db: &mut Arc<Mutex<Db>>,
        parse: &mut Parse
    ) -> crate::Result<Frame> {
        match Hset::parse_command(parse) {
            Ok(hset) => {
                let mut db = db.lock().unwrap();
                match db.get_dbtype_mut(&hset.key) {
                    Some(DbType::Hash(hash)) => {
                        // If the field exists, overwrite its value.
                        // 如果哈希表中已经包含该字段，覆盖其值
                        let is_new_field = hash.insert(hset.field.clone(), hset.value).is_none();
                        if is_new_field {
                            // If the field is newly added, return 1.
                            // 如果字段是新添加的，返回 1
                            Ok(Frame::Integer(1))
                        } else {
                            // If the field exists and is updated, return 0.
                            // 如果字段已存在并更新，返回 0
                            Ok(Frame::Integer(0))
                        }
                    },
                    // If the key does not exist, create a new hash and set the field and value.
                    // 如果哈希表不存在，创建新的哈希表，并设置字段和值
                    _ => {
                        let mut new_hash = HashMap::new();
                        new_hash.insert(hset.field, hset.value);
                        db.set(&hset.key, DbType::Hash(new_hash), None);
                        Ok(Frame::Integer(1)) // Return 1 as the new field is added.
                    }
                }
            }
            Err(_) => {
                // Incorrect number of arguments, return error. / 参数数量错误，返回错误。
                Ok(Frame::Error("ERR wrong number of arguments for 'hset' command".to_string()))
            }
        }
    }

    /// Parses the command and retrieves the key, field, and value for the hash.
    ///
    /// 解析命令并获取哈希表的键、字段和值。
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing the `Hset` struct with the parsed key, field, and value if successful.
    /// Otherwise, returns an error frame indicating the problem.
    ///
    /// 如果成功，返回包含解析后的键、字段和值的 `Hset` 结构体。如果失败，返回错误帧以指示问题。
    fn parse_command(parse: &mut Parse) -> crate::Result<Self> {
        // The command requires exactly three arguments: the key, field, and value.
        // 命令需要正好三个参数：键、字段和值。
        if parse.args_number()? != 3 {
            return Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, "ERR wrong number of arguments for 'hset' command")));
        }

        let key = parse.next_string()?;
        let field = parse.next_string()?;
        let value = parse.next_string()?;

        Ok(Hset {
            key,
            field,
            value,
        })
    }
}
