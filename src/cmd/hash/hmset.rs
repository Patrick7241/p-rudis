use std::sync::Arc;
use std::sync::Mutex;
use crate::db::{Db, DbType};
use crate::frame::Frame;
use crate::parse::Parse;
use std::collections::HashMap;

/// hash类型 hmset命令
/// 设置哈希表中多个字段的值。如果字段已存在，则覆盖它的值。
/// 返回 `OK` 表示命令执行成功。

pub struct Hmset {
    key: String,
    fields_values: Vec<(String, String)>,
}

impl Hmset {
    pub fn hmset_command(
        db: &mut Arc<Mutex<Db>>,
        parse: &mut Parse
    ) -> crate::Result<Frame> {
        match Hmset::parse_command(parse) {
            Ok(hmset) => {
                let mut db = db.lock().unwrap();
                match db.get_dbtype_mut(&hmset.key) {
                    Some(DbType::Hash(hash)) => {
                        // 遍历字段和值并插入或更新哈希表中的字段
                        for (field, value) in hmset.fields_values {
                            // 如果字段存在，更新其值；如果不存在，插入新字段
                            hash.insert(field, value);
                        }
                        Ok(Frame::Simple("OK".to_string()))
                    },
                    Some(_) => {
                        // 键存在，但类型不匹配，返回 WRONGTYPE 错误
                        Ok(Frame::Error("WRONGTYPE Operation against a key holding the wrong kind of value".to_string()))
                    },
                    None => {
                        // 键不存在，创建一个新的哈希表，并设置字段
                        let mut new_hash = HashMap::new();
                        for (field, value) in hmset.fields_values {
                            new_hash.insert(field, value);
                        }
                        db.set(&hmset.key, DbType::Hash(new_hash), None);
                        Ok(Frame::Simple("OK".to_string()))
                    }
                }
            }
            Err(_) => {
                Ok(Frame::Error("ERR wrong number of arguments for 'hmset' command".to_string()))
            }
        }
    }

    /// 解析命令并获取哈希表的字段和值
    fn parse_command(parse: &mut Parse) -> crate::Result<Self> {
        if parse.args_number()? < 3 || parse.args_number()? % 2 != 1 {
            return Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, "ERR wrong number of arguments for 'hmset' command")));
        }

        let key = parse.next_string()?;
        let mut fields_values = Vec::new();

        // 解析字段和值
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
