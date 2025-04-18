use std::sync::Arc;
use std::sync::Mutex;
use crate::db::{Db, DbType};
use crate::frame::Frame;
use crate::parse::Parse;

/// string类型 get命令

#[derive(Debug)]
pub struct Get {
    key: String,
}

impl Get {
    pub fn get_command(
        db: &mut Arc<Mutex<Db>>,
        parse: &mut Parse,
    ) -> crate::Result<Frame> {
        match Self::parse_command(parse) {
            Ok(get) => match db.lock().unwrap().get(&get.key) {
                // 返回Bulk类型
                Some(DbType::String(s)) =>{
                    Ok(Frame::Bulk(s.clone().into_bytes()))
                }
                // 返回错误类型
                Some(_) => Ok(Frame::Error("WRONGTYPE Operation against a key holding the wrong kind of value".to_string())),
                // 如果没有找到值，返回Null
                None => Ok(Frame::Null),
            },
            // 返回错误类型
            Err(_) =>{
                Ok(Frame::Error("ERR wrong number of arguments for 'get' command".to_string()))
            }
        }
    }

    /// 验证命令是否合法，并获取命令参数
    fn parse_command(parse: &mut Parse) -> crate::Result<Self> {
        let key = parse.next_string()?;

        Ok(Get { key })
    }

}
