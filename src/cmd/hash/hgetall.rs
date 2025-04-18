use std::sync::Arc;
use std::sync::Mutex;
use crate::db::{Db, DbType};
use crate::frame::Frame;
use crate::parse::Parse;

/// hash类型 hgetall命令
/// 获取哈希表中的所有字段和值。返回一个包含字段和值的数组。
/// 如果哈希表不存在，返回空数组。

pub struct Hgetall {
    key: String,
}

impl Hgetall {
    pub fn hgetall_command(
        db: &mut Arc<Mutex<Db>>,
        parse: &mut Parse
    ) -> crate::Result<Frame> {
        match Hgetall::parse_command(parse) {
            Ok(hgetall) => {
                let mut db = db.lock().unwrap();
                match db.get(&hgetall.key) {
                    Some(DbType::Hash(hash)) => {
                        let mut result = Vec::new();

                        // 遍历哈希表中的每个字段和值
                        for (field, value) in hash.iter() {
                            // 添加字段和值到返回结果中
                            result.push(Frame::Bulk(field.clone().into_bytes()));
                            result.push(Frame::Bulk(value.clone().into_bytes()));
                        }

                        // 返回结果，多个 Bulk 数据类型
                        Ok(Frame::Array(result))
                    },
                    Some(_) => {
                        // 键存在，但类型不匹配，返回 WRONGTYPE 错误
                        Ok(Frame::Error("WRONGTYPE Operation against a key holding the wrong kind of value".to_string()))
                    },
                    None => {
                        // 键不存在
                        Ok(Frame::Simple("(empty list or set)".to_string()))
                    }
                }
            }
            Err(_) => {
                Ok(Frame::Error("ERR wrong number of arguments for 'hgetall' command".to_string()))
            }
        }
    }

    /// 解析命令并获取哈希表的字段
    fn parse_command(parse: &mut Parse) -> crate::Result<Self> {
        if parse.args_number()? != 1 {
            return Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, "ERR wrong number of arguments for 'hgetall' command")));
        }
        let key = parse.next_string()?;

        Ok(Hgetall { key })
    }
}
