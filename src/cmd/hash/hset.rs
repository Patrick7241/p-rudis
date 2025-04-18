use std::collections::HashMap;
use std::sync::Arc;
use std::sync::Mutex;
use crate::db::{Db, DbType};
use crate::frame::Frame;
use crate::parse::Parse;

/// hash类型 hset命令
/// 设置哈希表中指定字段的值。如果字段已存在，则覆盖它的值。
/// 返回 `1` 表示字段已被新添加，返回 `0` 表示字段已被更新。

pub struct Hset {
    key: String,
    field: String,
    value: String,
}

impl Hset {
    pub fn hset_command(
        db: &mut Arc<Mutex<Db>>,
        parse: &mut Parse
    ) -> crate::Result<Frame> {
        match Hset::parse_command(parse) {
            Ok(hset) => {
                let mut db = db.lock().unwrap();
                match db.get_dbtype_mut(&hset.key) {
                    Some(DbType::Hash(hash)) => {
                        // 如果哈希表中已经包含该字段，覆盖其值
                        let is_new_field = hash.insert(hset.field.clone(), hset.value).is_none();
                        if is_new_field {
                            // 如果字段是新添加的，返回 1
                            Ok(Frame::Integer(1))
                        } else {
                            // 如果字段已存在并更新，返回 0
                            Ok(Frame::Integer(0))
                        }
                    },
                    // 如果哈希表不存在，创建新的哈希表，并设置字段和值
                    _ => {
                        let mut new_hash = HashMap::new();
                        new_hash.insert(hset.field, hset.value);
                        db.set(&hset.key, DbType::Hash(new_hash), None);
                        Ok(Frame::Integer(1)) // 新字段添加，返回 1
                    }
                }
            }
            Err(_) => {
                Ok(Frame::Error("ERR wrong number of arguments for 'hset' command".to_string()))
            }
        }
    }

    /// 解析命令并获取哈希表的字段和值对
    fn parse_command(parse: &mut Parse) -> crate::Result<Self> {
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
