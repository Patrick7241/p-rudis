use std::sync::Arc;
use std::sync::Mutex;
use crate::db::{Db, DbType};
use crate::frame::Frame;
use crate::parse::Parse;

/// string类型 incr命令
/// 将指定键的数值增加指定的步长，无默认值
/// 如果键不存在，新建一个键，值为 1
/// 返回增加后的新值

pub struct Incr {
    key: String,
    step: i64,
}

impl Incr {
    pub fn incr_command(
        db: &mut Arc<Mutex<Db>>,
        parse: &mut Parse
    ) -> crate::Result<Frame> {
        match Incr::parse_command(parse) {
            Ok(incr) => {
                let mut db = db.lock().unwrap();
                // 获取键的当前值
                match db.get(&incr.key) {
                    // 如果键存在且值为数字，进行增加
                    Some(DbType::String(value)) => {
                        match value.parse::<i64>() {
                            Ok(current_value) => {
                                let new_value = current_value + incr.step;
                                db.set(&incr.key, DbType::String(new_value.to_string()), None);
                                Ok(Frame::Integer(new_value))
                            }
                            // 键不为数字，返回错误
                            Err(_) => {
                                Ok(Frame::Error("ERR value is not an integer or out of range".to_string()))
                            }
                        }
                    }
                    // 如果键不存在，初始化为 step，然后增加
                    _ => {
                        let new_value = incr.step;
                        db.set(&incr.key, DbType::String(new_value.to_string()), None);
                        Ok(Frame::Integer(new_value))
                    }
                }
            }
            Err(_) => {
                Ok(Frame::Error("ERR wrong number of arguments for 'incr' command".to_string()))
            }
        }
    }

    /// 解析命令并获取参数
    fn parse_command(parse: &mut Parse) -> crate::Result<Self> {
        let key = parse.next_string()?;
        // 步长默认为1
        let step=1;

        Ok(Incr { key, step })
    }
}
