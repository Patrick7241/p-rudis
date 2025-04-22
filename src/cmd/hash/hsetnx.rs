use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use crate::db::{Db, DbType};
use crate::frame::Frame;
use crate::parse::Parse;
use crate::persistence::aof::propagate_aof;

/// Represents the `HSETNX` command in a Redis-like system.
/// `HSETNX` 命令仅在哈希表中指定字段不存在时才设置字段的值。
pub struct Hsetnx {
    key: String,   // The key of the hash in the database. / 数据库中哈希表的键。
    field: String, // The field to set in the hash. / 要在哈希表中设置的字段。
    value: String, // The value to associate with the field. / 要与字段关联的值。
}

impl Hsetnx {
    /// Executes the `HSETNX` command.
    /// 执行 `HSETNX` 命令。
    ///
    /// # Arguments
    /// - `db`: A mutable reference to the database (`Arc<Mutex<Db>>`), where the hash is stored.
    ///         / 数据库 (`Arc<Mutex<Db>>`) 的可变引用，存储哈希表的位置。
    /// - `parse`: A reference to the parser that contains the parsed command.
    ///            / 解析器的引用，包含解析后的命令。
    ///
    /// # Returns
    /// Returns an `Integer` frame with either `1` (if the field was added) or `0` (if the field already exists).
    /// 返回一个 `Integer` 类型的帧，值为 `1`（如果字段被添加）或 `0`（如果字段已存在）。
    pub fn hsetnx_command(
        db: &mut Arc<Mutex<Db>>,
        parse: &mut Parse,
    ) -> crate::Result<Frame> {
        match Hsetnx::parse_command(parse) {
            Ok(hsetnx) => {
                let mut db = db.lock().unwrap();
                match db.get_dbtype_mut(&hsetnx.key) {
                    Some(DbType::Hash(hash)) => {
                        // If the field exists, do nothing and return 0.
                        // 如果字段已存在，不做任何操作，返回 0。
                        if hash.contains_key(&hsetnx.field) {
                            Ok(Frame::Integer(0))
                        } else {
                            // If the field does not exist, insert it and return 1.
                            // 如果字段不存在，插入并返回 1。
                            hash.insert(hsetnx.field.clone(), hsetnx.value.clone());
                            // Propagate the command to AOF after successfully adding the field.
                            // 成功添加字段后将命令传播到 AOF。
                            Hsetnx::propagate_aof("hset", &hsetnx.key, &hsetnx.field, &hsetnx.value);
                            Ok(Frame::Integer(1))
                        }
                    },
                    // If the key does not exist, create a new hash and insert the field.
                    // 如果哈希表不存在，创建新的哈希表并插入字段，返回 1。
                    _ => {
                        let mut new_hash = HashMap::new();
                        new_hash.insert(hsetnx.field.clone(), hsetnx.value.clone());
                        db.set(&hsetnx.key, DbType::Hash(new_hash), None);
                        // Propagate the command to AOF after creating a new hash and adding the field.
                        // 创建新的哈希表并插入字段后将命令传播到 AOF。
                        Hsetnx::propagate_aof("hset", &hsetnx.key, &hsetnx.field, &hsetnx.value);
                        Ok(Frame::Integer(1)) // Return 1 as the new field is added.
                    }
                }
            }
            Err(_) => {
                // Incorrect number of arguments, return error. / 参数数量错误，返回错误。
                Ok(Frame::Error("ERR wrong number of arguments for 'hsetnx' command".to_string()))
            }
        }
    }

    /// Parses the command, ensuring there are exactly 3 arguments: key, field, and value.
    /// 解析命令，确保参数为 3 个：key，field，value。
    ///
    /// # Returns
    /// Returns a `Result` containing the `Hsetnx` struct with the parsed key, field, and value if successful.
    /// Otherwise, returns an error frame indicating the problem.
    /// 如果成功，返回包含解析后的键、字段和值的 `Hsetnx` 结构体。如果失败，返回错误帧以指示问题。
    fn parse_command(parse: &mut Parse) -> crate::Result<Self> {
        if parse.args_number()? != 3 {
            return Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, "ERR wrong number of arguments for 'hsetnx' command")));
        }
        let key = parse.next_string()?;
        let field = parse.next_string()?;
        let value = parse.next_string()?;

        Ok(Hsetnx {
            key,
            field,
            value,
        })
    }

    /// Propagates the `HSETNX` command to AOF.
    /// 将 `HSETNX` 命令传播到 AOF。
    fn propagate_aof(command: &str, key: &str, field: &str, value: &str) {
        // Propagate the HSETNX command with key, field, and value to AOF.
        // 将 `HSETNX` 命令与键、字段和值传播到 AOF。
        let args = vec![key.to_string(), field.to_string(), value.to_string()];
        propagate_aof(command.to_string(), args);
    }
}
