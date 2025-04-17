use std::sync::Arc;
use std::sync::Mutex;
use crate::db::{Db, DbType};
use crate::frame::Frame;
use crate::parse::Parse;

/// string类型 mget命令
/// 获取多个指定键的字符串值。如果键存在，则返回该键的值，如果不存在，则返回 `nil`。
/// 返回一个包含所有指定键的值的列表（如果某个键不存在，则为 `nil`）。

pub struct Mget {
    keys: Vec<String>,
}

impl Mget {
    pub fn mget_command(
        db: &mut Arc<Mutex<Db>>,
        parse: &mut Parse
    ) -> crate::Result<Frame> {
        match Mget::parse_command(parse) {
            Ok(mget) => {
                let mut db = db.lock().unwrap();

                let mut result = Vec::new();
                for key in mget.keys {
                    // 获取每个键的值
                    match db.get(&key) {
                        Some(DbType::String(value)) => {
                            result.push(Frame::Simple(value.to_string()));
                        }
                        _ => {
                            result.push(Frame::Null); // 键不存在或值非字符串
                        }
                    }
                }

                // 返回包含所有值的列表
                Ok(Frame::Array(result))
            }
            Err(_) => {
                Ok(Frame::Error("ERR wrong number of arguments for 'mget' command".to_string()))
            }
        }
    }

    /// 解析命令并获取参数
    fn parse_command(parse: &mut Parse) -> crate::Result<Self> {
        let mut keys = Vec::new();

        // 解析命令中的所有键
        while let Ok(key) = parse.next_string() {
            keys.push(key);
        }

        // 如果没有提供任何键，返回错误
        if keys.is_empty() {
            return Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, "ERR wrong number of arguments for 'mget' command")));
        }

        Ok(Mget { keys })
    }
}
