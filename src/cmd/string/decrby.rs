use std::sync::Arc;
use std::sync::Mutex;
use crate::db::{Db, DbType};
use crate::frame::Frame;
use crate::parse::Parse;

/// string类型 decrby命令
/// 将指定键的数值减少指定的步长，无默认值
/// 如果键不存在，新建一个键，值为 -step
/// 返回减少后的新值

pub struct DecrBy {
    key: String,
    step: i64,
}

impl DecrBy {
    pub fn decrby_command(
        db: &mut Arc<Mutex<Db>>,
        parse: &mut Parse
    ) -> crate::Result<Frame> {
        match DecrBy::parse_command(parse) {
            Ok(decr) => {
                let mut db = db.lock().unwrap();
                // 获取键的当前值
                match db.get(&decr.key) {
                    // 如果键存在且值为数字，进行减少
                    Some(DbType::String(value)) => {
                        match value.parse::<i64>() {
                            Ok(current_value) => {
                                let new_value = current_value - decr.step;
                                db.set(&decr.key, DbType::String(new_value.to_string()), None);
                                Ok(Frame::Integer(new_value))
                            }
                            // 键不为数字，返回错误
                            Err(_) => {
                                Ok(Frame::Error("ERR value is not an integer or out of range".to_string()))
                            }
                        }
                    }
                    // 如果键不存在，初始化为 -step
                    _ => {
                        let new_value = -decr.step;
                        db.set(&decr.key, DbType::String(new_value.to_string()), None);
                        Ok(Frame::Integer(new_value))
                    }
                }
            }
            Err(_) => {
                Ok(Frame::Error("ERR wrong number of arguments for 'decrby' command".to_string()))
            }
        }
    }

    /// 解析命令并获取参数
    fn parse_command(parse: &mut Parse) -> crate::Result<Self> {
        let key = parse.next_string()?;
        let step = parse.next_string()?;

        // 把step转成i64类型
        // 若转化失败返回错误
        let step: i64 = match step.parse() {
            Ok(num) => num,
            Err(_) => return Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, "ERR value is not an integer or out of range"))),
        };

        Ok(DecrBy { key, step })
    }
}
