use std::sync::{Arc, Mutex};
use crate::db::{Db, DbType};
use crate::frame::Frame;
use crate::parse::Parse;
use std::collections::HashMap;
use crate::persistence::aof::propagate_aof;

/// Represents the `HMSET` command in a Redis-like system.
/// `HMSET` 命令用于设置多个哈希表字段。
pub struct Hmset {
    key: String,  // The key of the hash in the database. / 数据库中哈希表的键。
    fields_values: Vec<(String, String)>,  // A vector of (field, value) pairs. / 字段和值对的向量。
}

impl Hmset {
    /// Executes the `HMSET` command.
    /// 执行 `HMSET` 命令。
    ///
    /// # Arguments
    /// - `db`: A mutable reference to the database (`Arc<Mutex<Db>>`).
    /// - `parse`: A reference to the parser that contains the parsed command.
    ///
    /// # Returns
    /// Returns `OK` indicating the success of the command.
    pub fn hmset_command(
        db: &mut Arc<Mutex<Db>>,
        parse: &mut Parse
    ) -> crate::Result<Frame> {
        match Hmset::parse_command(parse) {
            Ok(hmset) => {
                let mut db = db.lock().unwrap();
                match db.get_dbtype_mut(&hmset.key) {
                    Some(DbType::Hash(hash)) => {
                        // Iterate over the fields and values, and update or insert them in the hash.
                        // 遍历字段和值，并插入或更新哈希表中的字段。
                        for (field, value) in hmset.fields_values {
                            hash.insert(field.clone(), value.clone());
                            // Propagate each field-value pair to AOF after insertion.
                            // 插入后将每个字段-值对传播到 AOF。
                            Hmset::propagate_aof("hset", &hmset.key, &field, &value);
                        }
                        Ok(Frame::Simple("OK".to_string()))  // Return "OK" indicating success.
                    },
                    Some(_) => {
                        // If the key exists but it's not a hash, return WRONGTYPE error.
                        // 如果键存在但类型不匹配，返回 WRONGTYPE 错误。
                        Ok(Frame::Error("WRONGTYPE Operation against a key holding the wrong kind of value".to_string()))
                    },
                    None => {
                        // If the key does not exist, create a new hash and set the fields.
                        // 如果键不存在，创建新的哈希表，并设置字段。
                        let mut new_hash = HashMap::new();
                        for (field, value) in hmset.fields_values {
                            new_hash.insert(field.clone(), value.clone());
                            // Propagate each field-value pair to AOF after insertion.
                            // 插入后将每个字段-值对传播到 AOF。
                            Hmset::propagate_aof("hset", &hmset.key, &field, &value);
                        }
                        db.set(&hmset.key, DbType::Hash(new_hash), None); // Set the new hash in the database.
                        Ok(Frame::Simple("OK".to_string()))  // Return "OK" indicating success.
                    }
                }
            }
            Err(_) => {
                // If parsing fails, return an error indicating wrong number of arguments.
                // 如果解析失败，返回错误信息。
                Ok(Frame::Error("ERR wrong number of arguments for 'hmset' command".to_string()))
            }
        }
    }

    /// Parses the command and retrieves the key and fields/values for the hash.
    /// 解析命令并获取哈希表的键和字段/值。
    ///
    /// # Returns
    /// Returns a `Result` containing the `Hmset` struct with the parsed key and fields/values.
    /// 如果成功，返回包含解析后的键和字段/值的 `Hmset` 结构体。
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

    /// Propagates the `HMSET` command to AOF.
    /// 将 `HMSET` 命令传播到 AOF。
    fn propagate_aof(command: &str, key: &str, field: &str, value: &str) {
        // Propagate the field-value pair for each field in the hmset operation.
        // 对于 `hmset` 操作中的每个字段-值对，传播到 AOF。
        let args = vec![key.to_string(), field.to_string(), value.to_string()];
        propagate_aof(command.to_string(), args);
    }
}
