use std::sync::Arc;
use std::sync::Mutex;
use crate::db::{Db, DbType};
use crate::frame::Frame;
use crate::parse::Parse;

/// string类型 decr命令
/// 将指定键的数值减少1。
/// 如果键不存在，新建一个键，值为 -1
/// 返回减少后的新值

pub struct Decr {
    key: String,
}

impl Decr {
    pub fn decr_command(
        db: &mut Arc<Mutex<Db>>,
        parse: &mut Parse
    ) -> crate::Result<Frame> {
        match Decr::parse_command(parse) {
            Ok(decr) => {
                let mut db = db.lock().unwrap();
                // 获取键的当前值
                match db.get(&decr.key) {
                    // 如果键存在且值为数字，进行减少
                    Some(DbType::String(value)) => {
                        match value.parse::<i64>() {  // 允许负数
                            Ok(current_value) => {
                                let new_value = current_value - 1; // 减少 1
                                db.set(&decr.key, DbType::String(new_value.to_string()), None);
                                Ok(Frame::Integer(new_value))
                            }
                            // 键不为数字，返回错误
                            Err(_) => {
                                Ok(Frame::Error("ERR value is not an integer or out of range".to_string()))
                            }
                        }
                    }
                    // 如果键不存在，初始化为 -1，然后减少
                    _ => {
                        let new_value = -1;
                        db.set(&decr.key, DbType::String(new_value.to_string()), None);
                        Ok(Frame::Integer(new_value))
                    }
                }
            }
            Err(_) => {
                Ok(Frame::Error("ERR wrong number of arguments for 'decr' command".to_string()))
            }
        }
    }

    /// 解析命令并获取参数
    fn parse_command(parse: &mut Parse) -> crate::Result<Self> {
        let key = parse.next_string()?;
        Ok(Decr { key })
    }
}
