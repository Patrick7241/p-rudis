use std::sync::{Arc, Mutex};
use crate::db::{Db, DbType};
use crate::frame::Frame;
use crate::parse::Parse;

/// string类型 msetnx命令
/// 设置多个键的值。只有当所有键都不存在时，才会执行设置操作；
/// 如果有任何一个键已经存在，则操作不执行，返回 `0`。
/// 如果所有键都不存在，则设置键值对并返回 `1`，表示操作成功。
/// 返回 `0` 表示至少有一个键已经存在，操作未执行。


pub struct Msetnx {
    keys_values: Vec<(String, String)>,
}

impl Msetnx {
    pub fn msetnx_command(
        db: &mut Arc<Mutex<Db>>,
        parse: &mut Parse,
    ) -> crate::Result<Frame> {
        match Msetnx::parse_command(parse) {
            Ok(msetnx) => {
                let mut db = db.lock().unwrap();

                // 检查所有键是否存在
                for (key, _) in &msetnx.keys_values {
                    if db.exists(key) {
                        // 如果有任何一个键已经存在，返回 0
                        return Ok(Frame::Integer(0));
                    }
                }

                // 如果所有键都不存在，则设置它们
                for (key, value) in msetnx.keys_values {
                    db.set(&key, DbType::String(value), None);
                }

                // 设置成功，返回 1
                Ok(Frame::Integer(1))
            }
            Err(_) => {
                Ok(Frame::Error("ERR wrong number of arguments for 'msetnx' command".to_string()))
            }
        }
    }

    fn parse_command(parse: &mut Parse) -> crate::Result<Self> {
        let mut keys_values = Vec::new();

        while let Ok(key) = parse.next_string() {
            let value = parse.next_string()?;
            keys_values.push((key, value));
        }
        if keys_values.is_empty() || keys_values.len() % 2 != 0 {
            return Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "ERR wrong number of arguments for 'msetnx' command",
            )));
        }

        Ok(Msetnx { keys_values })
    }
}
