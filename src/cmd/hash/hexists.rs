use std::sync::Arc;
use std::sync::Mutex;
use crate::db::{Db, DbType};
use crate::frame::Frame;
use crate::parse::Parse;

/// hash类型 hexists命令
/// 检查哈希表中指定字段是否存在。返回 1 表示字段存在，返回 0 表示字段不存在。
/// 返回值为 `Integer` 类型。

pub struct Hexists {
    key: String,
    field: String,
}

impl Hexists {
    pub fn hexists_command(
        db: &mut Arc<Mutex<Db>>,
        parse: &mut Parse
    ) -> crate::Result<Frame> {
        match Hexists::parse_command(parse) {
            Ok(hexists) => {
                let mut db = db.lock().unwrap();
                match db.get(&hexists.key) {
                    Some(DbType::Hash(hash)) => {
                        // 检查字段是否存在
                        if hash.contains_key(&hexists.field) {
                            Ok(Frame::Integer(1)) // 字段存在，返回 1
                        } else {
                            Ok(Frame::Integer(0)) // 字段不存在，返回 0
                        }
                    },
                    Some(_) => {
                        // 键存在，但类型不匹配，返回 WRONGTYPE 错误
                        Ok(Frame::Error("WRONGTYPE Operation against a key holding the wrong kind of value".to_string()))
                    },
                    None => {
                        // 键不存在，返回 0
                        Ok(Frame::Integer(0))
                    }
                }
            }
            Err(_) => {
                Ok(Frame::Error("ERR wrong number of arguments for 'hexists' command".to_string()))
            }
        }
    }

    /// 解析命令并获取哈希表的字段
    fn parse_command(parse: &mut Parse) -> crate::Result<Self> {
        if parse.args_number()? != 2 {
            return Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, "ERR wrong number of arguments for 'hexists' command")));
        }

        let key = parse.next_string()?;
        let field = parse.next_string()?;

        Ok(Hexists { key, field })
    }
}
