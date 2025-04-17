use std::sync::Arc;
use std::sync::Mutex;
use crate::db::Db;
use crate::frame::Frame;
use crate::parse::Parse;

/// string类型 del命令
/// 删除指定的多个键，返回成功删除的键的数量
pub struct Del {
    keys: Vec<String>,
}

impl Del {
    pub fn del_command(
        db: &mut Arc<Mutex<Db>>,
        parse: &mut Parse
    ) -> crate::Result<Frame> {
        match Del::parse_command(parse) {
            Ok(del) => {
                let mut db = db.lock().unwrap();
                let mut deleted_count = 0;

                // 遍历所有键，尝试删除它们
                for key in del.keys {
                    if db.del(&key) {
                        deleted_count += 1;
                    }
                }

                // 返回删除的键的数量
                Ok(Frame::Integer(deleted_count))
            }
            Err(_) => {
                Ok(Frame::Error("ERR wrong number of arguments for 'del' command".to_string()))
            }
        }
    }

    /// 验证命令是否合法，并获取命令参数
    fn parse_command(parse: &mut Parse) -> crate::Result<Self> {
        let mut keys = Vec::new();

        // 一直到没有更多的键为止
        while let Ok(key) = parse.next_string() {
            keys.push(key);
        }

        // 如果没有传入键，返回错误
        if keys.is_empty() {
            return Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                "ERR wrong number of arguments for 'del' command"
            )));
        }

        Ok(Del { keys })
    }
}
