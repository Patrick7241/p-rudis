use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use crate::db::{Db, DbType};
use crate::frame::Frame;
use crate::parse::Parse;
use crate::persistence::aof::propagate_aof;

/// Represents the `HSET` command in a Redis-like system.
/// `HSET` 命令在 Redis 风格的系统中设置哈希表字段的值。
pub struct Hset {
    key: String,   // The key of the hash in the database. / 数据库中哈希表的键
    field: String, // The field to set in the hash. / 要在哈希表中设置的字段
    value: String, // The value to associate with the field. / 要与字段关联的值
}

impl Hset {
    /// Executes the `HSET` command.
    /// 执行 `HSET` 命令，设置指定哈希表中的字段值。
    pub fn hset_command(db: &mut Arc<Mutex<Db>>, parse: &mut Parse) -> crate::Result<Frame> {
        match Hset::parse_command(parse) {
            Ok(hset) => {
                let mut db = db.lock().unwrap();

                // Try to get the existing hash, or create a new one if it doesn't exist.
                match db.get_dbtype_mut(&hset.key) {
                    Some(DbType::Hash(hash)) => {
                        // Insert the field and value into the existing hash.
                        // 判断字段是否为新添加
                        let is_new_field = hash.insert(hset.field.clone(), hset.value.clone()).is_none();
                        Hset::propagate_aof("hset", &hset.key, &hset.field, &hset.value);
                        Ok(Frame::Integer(if is_new_field { 1 } else { 0 }))
                    }
                    _ => {
                        // If the key doesn't exist, create a new hash.
                        let mut new_hash = HashMap::new();
                        new_hash.insert(hset.field.clone(), hset.value.clone());
                        db.set(&hset.key, DbType::Hash(new_hash), None);
                        Hset::propagate_aof("hset", &hset.key, &hset.field, &hset.value);
                        Ok(Frame::Integer(1)) // Return 1 for newly added field.
                    }
                }
            }
            Err(_) => Ok(Frame::Error("ERR wrong number of arguments for 'hset' command".to_string())),
        }
    }

    /// Parses the command and retrieves the key, field, and value for the hash.
    /// 解析命令并获取哈希表的键、字段和值。
    fn parse_command(parse: &mut Parse) -> crate::Result<Self> {
        if parse.args_number()? != 3 {
            return Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                "ERR wrong number of arguments for 'hset' command",
            )));
        }

        let key = parse.next_string()?;
        let field = parse.next_string()?;
        let value = parse.next_string()?;

        Ok(Hset { key, field, value })
    }

    /// Helper function to propagate the `hset` command to AOF.
    /// 辅助函数，将 `hset` 命令传播到 AOF。
    fn propagate_aof(command: &str, key: &str, field: &str, value: &str) {
        let args = vec![key.to_string(), field.to_string(), value.to_string()];
        propagate_aof(command.to_string(), args);
    }
}
