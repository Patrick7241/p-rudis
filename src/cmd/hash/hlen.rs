use std::sync::Arc;
use std::sync::Mutex;
use crate::db::{Db, DbType};
use crate::frame::Frame;
use crate::parse::Parse;

/// hash类型 hlen命令
/// 获取哈希表中字段的数量。如果哈希表不存在，则返回 `nil`。
/// 返回值为一个整数，表示哈希表中的字段数量。

pub struct Hlen {
    key: String,
}

impl Hlen {
    pub fn hlen_command(
        db: &mut Arc<Mutex<Db>>,
        parse: &mut Parse
    ) -> crate::Result<Frame> {
        match Hlen::parse_command(parse) {
            Ok(hlen) => {
                let mut db = db.lock().unwrap();
                match db.get(&hlen.key) {
                    Some(DbType::Hash(hash)) => {
                        // 获取哈希表中的字段数量
                        let field_count = hash.len();
                        // 返回字段数量
                        Ok(Frame::Integer(field_count as i64))
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
                Ok(Frame::Error("ERR wrong number of arguments for 'hlen' command".to_string()))
            }
        }
    }

    /// 解析命令并获取哈希表的字段
    fn parse_command(parse: &mut Parse) -> crate::Result<Self> {
        if parse.args_number()? != 1 {
            return Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, "ERR wrong number of arguments for 'hlen' command")));
        }

        let key = parse.next_string()?;

        Ok(Hlen { key })
    }
}
