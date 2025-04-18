use std::sync::Arc;
use std::sync::Mutex;
use crate::db::{Db, DbType};
use crate::frame::Frame;
use crate::parse::Parse;

/// hash类型 hget命令
/// 获取哈希表中指定字段的值。如果字段不存在，返回 `nil`。
/// 返回值为 `Bulk`，包含字段的值。

pub struct Hget {
    key: String,
    field: String,
}

impl Hget {
    pub fn hget_command(
        db: &mut Arc<Mutex<Db>>,
        parse: &mut Parse
    ) -> crate::Result<Frame> {
        match Hget::parse_command(parse) {
            Ok(hget) => {
                let mut db = db.lock().unwrap();
                match db.get(&hget.key) {
                    Some(DbType::Hash(hash)) => {
                        // 如果字段存在，则返回其值
                        if let Some(value) = hash.get(&hget.field) {
                            Ok(Frame::Bulk(value.clone().into_bytes())) // 返回字段的值
                        } else {
                            // 如果字段不存在，返回 nil
                            Ok(Frame::Null)
                        }
                    },
                    Some(_) => {
                        // 键存在，但类型不匹配，返回 WRONGTYPE 错误
                        Ok(Frame::Error("WRONGTYPE Operation against a key holding the wrong kind of value".to_string()))
                    },
                    None => {
                        // 键不存在，返回 nil
                        Ok(Frame::Null)
                    }
                }
            }
            Err(_) => {
                Ok(Frame::Error("ERR wrong number of arguments for 'hget' command".to_string()))
            }
        }
    }

    /// 解析命令并获取哈希表的字段
    fn parse_command(parse: &mut Parse) -> crate::Result<Self> {
        if parse.args_number()? != 2 {
            return Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, "ERR wrong number of arguments for 'hget' command")));
        }
        let key = parse.next_string()?;
        let field = parse.next_string()?;

        Ok(Hget { key, field })
    }
}
