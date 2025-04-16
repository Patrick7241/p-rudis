use std::io::Error;
use crate::db::{Db, DbType};
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
    ) -> crate::Result<String> {
        match Self::parse_command(parse) {
            Ok(get) => match db.get(&get.key) {
                Some(DbType::String(s)) => Ok(s.clone()), // 避免不必要的 `to_string()` 调用
                Some(_) => Ok("类型转换失败".to_string()),
                None => Ok("nil".to_string()),
            },
            Err(err) => Ok(err.to_string()),
        }
    }

    /// 验证命令是否合法，并获取命令参数
    fn parse_command(parse: &mut Parse) -> crate::Result<Self> {
        let key = parse.next_string()?.to_lowercase();
        if key.is_empty() {
            return Err(Box::new(Error::new(
                std::io::ErrorKind::InvalidInput,
                "'get' 命令需要至少一个参数",
            )));
        }

        Ok(Get { key })
    }
}
