use std::sync::Arc;
use std::sync::Mutex;
use crate::db::{Db, DbType};
use crate::frame::Frame;
use crate::parse::Parse;

/// hash类型 hmget命令
/// 获取哈希表中多个字段的值。字段不存在则返回 `nil`。
/// 返回值为一个 Bulk 数组，每个字段的值为 `Bulk`，如果字段不存在，则为 `Null`。

pub struct Hmget {
    key: String,
    fields: Vec<String>,
}

impl Hmget {
    pub fn hmget_command(
        db: &mut Arc<Mutex<Db>>,
        parse: &mut Parse
    ) -> crate::Result<Frame> {
        match Hmget::parse_command(parse) {
            Ok(hmget) => {
                let mut db = db.lock().unwrap();
                match db.get(&hmget.key) {
                    Some(DbType::Hash(hash)) => {
                        // 为每个字段获取值
                        let mut result = Vec::new();
                        for field in hmget.fields {
                            if let Some(value) = hash.get(&field) {
                                // 如果字段存在，返回字段的值
                                result.push(Frame::Bulk(value.clone().into_bytes()));
                            } else {
                                // 如果字段不存在，返回 nil
                                result.push(Frame::Null);
                            }
                        }

                        // 返回字段值的列表
                        Ok(Frame::Array(result))
                    },
                    Some(_) => {
                        // 键存在，但类型不匹配，返回 WRONGTYPE 错误
                        Ok(Frame::Error("WRONGTYPE Operation against a key holding the wrong kind of value".to_string()))
                    },
                    None => {
                        // 键不存在，返回 nil
                        let result = vec![Frame::Null; hmget.fields.len()];
                        Ok(Frame::Array(result))
                    }
                }
            }
            Err(_) => {
                Ok(Frame::Error("ERR wrong number of arguments for 'hmget' command".to_string()))
            }
        }
    }

    /// 解析命令并获取哈希表的字段
    fn parse_command(parse: &mut Parse) -> crate::Result<Self> {
        if parse.args_number()? < 2 {
            return Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, "ERR wrong number of arguments for 'hmget' command")));
        }

        let key = parse.next_string()?;
        let mut fields = Vec::new();

        // 解析字段
        while let Ok(field) = parse.next_string() {
            fields.push(field);
        }

        Ok(Hmget { key, fields })
    }
}
