use std::io::Error;
use crate::db::{Db, DbType};
use crate::frame::Frame;
use crate::parse::Parse;

#[derive(Debug)]
pub struct Get {
    key: String,
}

impl Get {
    /// get命令
    pub fn get_command(
        db: &mut Db,
        parse: &mut Parse,
    ) -> crate::Result<Frame> {
        match Self::parse_command(parse) {
            Ok(get) => match db.get(&get.key) {
                // 返回Bulk类型
                Some(DbType::String(s)) => Ok(Frame::Bulk(s.clone().into_bytes())),
                // 返回错误类型
                Some(_) => Ok(Frame::Error("ERR type conversion failed".to_string())),
                // 如果没有找到值，返回Null
                None => Ok(Frame::Null),
            },
            // 返回错误类型
            Err(err) => Ok(Frame::Error(err.to_string())),
        }
    }

    /// 验证命令是否合法，并获取命令参数
    fn parse_command(parse: &mut Parse) -> crate::Result<Self> {
        let key = parse.next_string()?.to_lowercase();
        if key.is_empty() {
            return Err(Box::new(Error::new(
                std::io::ErrorKind::InvalidInput,
                "ERR wrong number of arguments for 'get' command",
            )));
        }

        Ok(Get { key })
    }
}
