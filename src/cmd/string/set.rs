use std::io::Error;
use std::sync::Arc;
use std::sync::Mutex;
use crate::db::{Db, DbType};
use crate::frame::Frame;
use crate::parse::Parse;

/// string类型 set命令
/// 支持 EX PX 设置过期时间，EX单位是秒，PX单位是毫秒
/// 支持 NX XX NX表示只有当键不存在时才设置，XX表示只有当键存在时才设置

pub struct Set {
    key: String,
    value: String,
    expiration: Option<u64>, // 单位：毫秒
    nx: bool,
    xx: bool,
}
impl Set {
    pub fn set_command(
        db: &mut Arc<Mutex<Db>>,
        parse: &mut Parse
    ) -> crate::Result<Frame> {
        match Set::parse_command(parse) {
            Ok(set) => {
                let mut db = db.lock().unwrap();
                // 检查 NX/XX 条件
                let exists = db.exists(&set.key);
                if (set.nx && exists) || (set.xx && !exists) {
                    return Ok(Frame::Null);
                }

                // 设置键值
                db.set(&set.key, DbType::String(set.value), set.expiration);

                // 返回成功响应
                Ok(Frame::Simple("OK".to_string()))
            }
            Err(_) => {
                Ok(Frame::Error("ERR wrong number of arguments for 'set' command".to_string()))
            }
        }
    }
    /// 验证命令是否合法，并获取命令参数
    fn parse_command(parse: &mut Parse) -> crate::Result<Self> {
        let key = parse.next_string()?;
        let value = parse.next_string()?;

        let mut expiration = None;
        let mut nx = false;
        let mut xx = false;

        // 解析可选的 EX PX NX XX 参数
        while let Ok(option) = parse.next_string() {
            match option.to_uppercase().as_str() {
                "EX" => {
                    // EX 后面应跟数字（秒）
                    let exp = parse.next_string()?;
                    let exp_in_sec: u64 = exp.parse()?;
                    // 将秒统一转换为毫秒
                    expiration = Some(exp_in_sec * 1000);
                }
                "PX" => {
                    // PX 后面应跟数字（毫秒）
                    let exp = parse.next_string()?;
                    let exp_in_ms: u64 = exp.parse()?;
                    // 直接使用毫秒值
                    expiration = Some(exp_in_ms);
                }
                "NX" => {
                    if xx {
                        return Err(Box::new(Error::new(std::io::ErrorKind::Other, "ERR syntax error")));
                    }
                    nx = true;
                }
                "XX" => {
                    if nx {
                        return Err(Box::new(Error::new(std::io::ErrorKind::Other, "ERR syntax error")));
                    }
                    xx = true;
                }
                _ => break, // 遇到未知参数时，停止解析
            }
        }

        // 返回构造好的 Set 结构
        Ok(Set {
            key,
            value,
            expiration,
            nx,
            xx,
        })
    }
}
