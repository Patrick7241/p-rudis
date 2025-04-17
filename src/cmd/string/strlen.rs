use std::sync::Arc;
use std::sync::Mutex;
use crate::db::{Db, DbType};
use crate::frame::Frame;
use crate::parse::Parse;

/// string类型 strlen命令
/// 获取指定键的字符串值的长度。如果键不存在，返回 0

pub struct Strlen {
    key: String,
}

impl Strlen {
    pub fn strlen_command(
        db: &mut Arc<Mutex<Db>>,
        parse: &mut Parse
    ) -> crate::Result<Frame> {
        match Strlen::parse_command(parse) {
            Ok(strlen) => {
                let mut db = db.lock().unwrap();
                // 获取键的当前值
                match db.get(&strlen.key) {
                    // 如果键存在并且是字符串类型，返回其长度
                    Some(DbType::String(value)) => Ok(Frame::Integer(value.len() as i64)),
                    // 如果键不存在或不是字符串类型，返回 0
                    _ => Ok(Frame::Integer(0)),
                }
            }
            Err(_) => {
                Ok(Frame::Error("ERR wrong number of arguments for 'strlen' command".to_string()))
            }
        }
    }

    /// 解析命令并获取参数
    fn parse_command(parse: &mut Parse) -> crate::Result<Self> {
        let key = parse.next_string()?;
        Ok(Strlen { key })
    }
}
