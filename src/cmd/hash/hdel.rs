use std::sync::Arc;
use std::sync::Mutex;
use crate::db::{Db, DbType};
use crate::frame::Frame;
use crate::parse::Parse;

/// hash类型 hdel命令
/// 删除哈希表中的一个或多个字段。返回被删除字段的数量。
/// 如果字段不存在，返回 `0`。

pub struct Hdel {
    key: String,
    fields: Vec<String>,
}

impl Hdel {
    pub fn hdel_command(
        db: &mut Arc<Mutex<Db>>,
        parse: &mut Parse
    ) -> crate::Result<Frame> {
        match Hdel::parse_command(parse) {
            Ok(hdel) => {
                let mut db = db.lock().unwrap();
                match db.get_dbtype_mut(&hdel.key) {
                    Some(DbType::Hash( hash)) => {
                        // 统计删除的字段数
                        let mut deleted_count = 0;

                        // 删除字段并计数
                        for field in &hdel.fields {
                            if hash.remove(field).is_some() {
                                deleted_count += 1;
                            }
                        }

                        // 返回被删除字段的数量
                        Ok(Frame::Integer(deleted_count as i64))
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
                Ok(Frame::Error("ERR wrong number of arguments for 'hdel' command".to_string()))
            }
        }
    }

    /// 解析命令并获取哈希表的字段
    fn parse_command(parse: &mut Parse) -> crate::Result<Self> {
        // 需要的最少参数为 2，即键和至少一个字段
        let args_number = parse.args_number()?;
        if args_number < 2 {
            return Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, "ERR wrong number of arguments for 'hdel' command")));
        }

        let key = parse.next_string()?;
        let mut fields = Vec::with_capacity(args_number - 1);

        // 获取所有字段
        for _ in 0..(args_number - 1) {
            let field = parse.next_string()?;
            fields.push(field);
        }

        Ok(Hdel { key, fields })
    }
}
