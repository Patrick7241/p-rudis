use std::collections::HashMap;
use std::sync::Arc;
use std::sync::Mutex;
use crate::db::{Db, DbType};
use crate::frame::Frame;
use crate::parse::Parse;

/// hash类型 hsetnx命令
/// 设置哈希表中指定字段的值，仅当该字段不存在时才设置。
/// 返回 `1` 表示字段被添加成功，返回 `0` 表示字段已存在，未进行任何操作。

pub struct Hsetnx {
    key: String,
    field: String,
    value: String,
}

impl Hsetnx {
    pub fn hsetnx_command(
        db: &mut Arc<Mutex<Db>>,
        parse: &mut Parse,
    ) -> crate::Result<Frame> {
        match Hsetnx::parse_command(parse) {
            Ok(hsetnx) => {
                let mut db = db.lock().unwrap();
                match db.get_dbtype_mut(&hsetnx.key) {
                    Some(DbType::Hash(hash)) => {
                        // 如果字段已存在，不做任何操作，返回 0
                        if hash.contains_key(&hsetnx.field) {
                            Ok(Frame::Integer(0))
                        } else {
                            // 字段不存在，插入并返回 1
                            hash.insert(hsetnx.field, hsetnx.value);
                            Ok(Frame::Integer(1))
                        }
                    },
                    // 如果哈希表不存在，创建并插入字段，返回 1
                    _ => {
                        let mut new_hash = HashMap::new();
                        new_hash.insert(hsetnx.field, hsetnx.value);
                        db.set(&hsetnx.key, DbType::Hash(new_hash), None);
                        Ok(Frame::Integer(1))
                    }
                }
            }
            Err(_) => {
                Ok(Frame::Error("ERR wrong number of arguments for 'hsetnx' command".to_string()))
            }
        }
    }

    /// 解析命令，确保参数为3个：key field value
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
}
