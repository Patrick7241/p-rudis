use std::sync::Arc;
use std::sync::Mutex;
use crate::db::{Db, DbType};
use crate::frame::Frame;
use crate::parse::Parse;

/// hash类型 hvals命令
/// 获取哈希表中所有字段的值。如果哈希表不存在，则返回 `nil`。
/// 返回值为一个 `Array`，包含哈希表中的所有字段值。

pub struct Hvals {
    key: String,
}

impl Hvals {
    pub fn hvals_command(
        db: &mut Arc<Mutex<Db>>,
        parse: &mut Parse
    ) -> crate::Result<Frame> {
        match Hvals::parse_command(parse) {
            Ok(hvals) => {
                let mut db = db.lock().unwrap();
                match db.get(&hvals.key) {
                    Some(DbType::Hash(hash)) => {
                        // 获取所有的字段值
                        let values: Vec<Frame> = hash.values()
                            .map(|value| Frame::Bulk(value.clone().into_bytes())) // 将字段值转化为 Frame::Bulk
                            .collect();

                        // 返回字段值的数组
                        Ok(Frame::Array(values))
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
                Ok(Frame::Error("ERR wrong number of arguments for 'hvals' command".to_string()))
            }
        }
    }

    /// 解析命令并获取哈希表的字段
    fn parse_command(parse: &mut Parse) -> crate::Result<Self> {
        if parse.args_number()? != 1 {
            return Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, "ERR wrong number of arguments for 'hvals' command")));
        }

        let key = parse.next_string()?;

        Ok(Hvals { key })
    }
}
