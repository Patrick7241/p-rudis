use std::sync::Arc;
use std::sync::Mutex;
use crate::db::{Db, DbType};
use crate::frame::Frame;
use crate::parse::Parse;

/// string类型 mset命令
/// 设置多个键的值。如果某个键已经存在，则覆盖它的值。
/// 返回 `OK` 表示命令执行成功。

pub struct Mset {
    keys_values: Vec<(String, String)>,
}

impl Mset {
    pub fn mset_command(
        db: &mut Arc<Mutex<Db>>,
        parse: &mut Parse
    ) -> crate::Result<Frame> {
        match Mset::parse_command(parse) {
            Ok(mset) => {
                let mut db = db.lock().unwrap();

                // 遍历键值对，设置每个键的值
                for (key, value) in mset.keys_values {
                    db.set(&key, DbType::String(value), None);
                }

                // 返回成功响应
                Ok(Frame::Simple("OK".to_string()))
            }
            Err(_) => {
                Ok(Frame::Error("ERR wrong number of arguments for 'mset' command".to_string()))
            }
        }
    }

    /// 解析命令并获取键值对
    fn parse_command(parse: &mut Parse) -> crate::Result<Self> {
        let mut keys_values = Vec::new();

        // 解析命令中的键值对
        while let Ok(key) = parse.next_string() {
            let value = parse.next_string()?;
            keys_values.push((key, value));
        }

        // 如果键值对的数量不为偶数，返回错误
        if keys_values.is_empty() || keys_values.len() % 2 != 0 {
            return Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, "ERR wrong number of arguments for 'mset' command")));
        }

        Ok(Mset { keys_values })
    }
}
