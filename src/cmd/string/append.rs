use std::sync::Arc;
use std::sync::Mutex;
use crate::connection::ConnectionHandler;
use crate::db::{Db, DbType};
use crate::frame::Frame;
use crate::parse::Parse;

/// string类型 append命令
/// 将指定的值追加到键的字符串值后面。如果键不存在，创建一个新键并设置其值为指定的值。
/// 返回追加后的新字符串的长度。

pub struct Append {
    key: String,
    value: String,
}

impl Append {
    pub fn append_command(
        db: &mut Arc<Mutex<Db>>,
        parse: &mut Parse,
    ) -> crate::Result<Frame> {
        match Append::parse_command(parse) {
            Ok(append) => {
                let mut db = db.lock().unwrap();
                // 获取键当前的值
                let current_value = db.get(&append.key);

                let new_value = match current_value {
                    // 如果键存在，追加新的值
                    Some(DbType::String(existing_value)) => {
                        format!("{}{}", existing_value, append.value)
                    },
                    // 如果键不存在，设置新的值
                    _ => append.value.clone(),
                };

                // 设置或更新键的值
                db.set(&append.key, DbType::String(new_value.clone()), None);

                // 返回追加后的新值的长度
                Ok(Frame::Integer(new_value.len() as i64))
            }
            Err(_) => {
                Ok(Frame::Error("ERR wrong number of arguments for 'append' command".to_string()))
            }
        }
    }

    /// 解析命令并获取参数
    fn parse_command(parse: &mut Parse) -> crate::Result<Self> {
        let key = parse.next_string()?;
        let value = parse.next_string()?;

        Ok(Append { key, value })
    }
}
