use std::io::Error;
use std::sync::Arc;
use std::sync::Mutex;
use crate::db::{Db, DbType};
use crate::frame::Frame;
use crate::parse::Parse;

/// `Set` command for string type.
/// `Set` 命令用于字符串类型。
///
/// Sets the value of a key. It supports expiration time with `EX` (in seconds) or `PX` (in milliseconds).
/// 支持设置键的值。它支持过期时间，使用 `EX`（秒）或 `PX`（毫秒）表示过期时间。
/// It also supports `NX` (set only if key does not exist) and `XX` (set only if key exists).
/// 它还支持 `NX`（只有在键不存在时设置）和 `XX`（只有在键存在时设置）。
pub struct Set {
    key: String,           // The key to set / 要设置的键
    value: String,         // The value to set / 要设置的值
    expiration: Option<u64>, // Expiration time in milliseconds / 过期时间，单位：毫秒
    nx: bool,              // Whether to set only if the key does not exist / 是否只有在键不存在时才设置
    xx: bool,              // Whether to set only if the key exists / 是否只有在键存在时才设置
}

impl Set {
    /// Executes the `set` command.
    /// 执行 `set` 命令。
    ///
    /// # Arguments
    /// - `db`: Shared reference to the database for access. / 用于访问数据库的共享引用。
    /// - `parse`: For parsing the command from the client. / 用于解析客户端传来的命令。
    ///
    /// # Return
    /// - Returns `"OK"` if the command is successful. / 如果命令成功，返回 `"OK"`。
    /// - Returns `NULL` if the `NX` or `XX` condition is not met. / 如果不满足 `NX` 或 `XX` 条件，返回 `NULL`。
    pub fn set_command(
        db: &mut Arc<Mutex<Db>>,
        parse: &mut Parse
    ) -> crate::Result<Frame> {
        match Set::parse_command(parse) {
            Ok(set) => {
                let mut db = db.lock().unwrap();
                // Check NX/XX conditions
                // 检查 NX/XX 条件
                let exists = db.exists(&set.key);
                if (set.nx && exists) || (set.xx && !exists) {
                    return Ok(Frame::Null);
                }

                // Set the key-value pair
                // 设置键值对
                db.set(&set.key, DbType::String(set.value), set.expiration);

                // Return success response
                // 返回成功响应
                Ok(Frame::Simple("OK".to_string()))
            }
            // Return error if the command has incorrect number of arguments
            // 如果命令的参数数量不正确，返回错误
            Err(_) => {
                Ok(Frame::Error("ERR wrong number of arguments for 'set' command".to_string()))
            }
        }
    }

    /// Parses the command and retrieves the key-value pair and options.
    /// 解析命令并获取键值对和选项。
    ///
    /// # Arguments
    /// - `parse`: The `Parse` instance used to parse the command. / 用于解析命令的 `Parse` 实例。
    ///
    /// # Return
    /// - Returns the parsed `Set` instance with key, value, and options. / 返回解析后的 `Set` 实例，包含键、值和选项。
    fn parse_command(parse: &mut Parse) -> crate::Result<Self> {
        let key = parse.next_string()?;    // The key to set / 要设置的键
        let value = parse.next_string()?;  // The value to set / 要设置的值

        let mut expiration = None; // Expiration time (in milliseconds) / 过期时间（毫秒）
        let mut nx = false;        // `NX` flag / `NX` 标志
        let mut xx = false;        // `XX` flag / `XX` 标志

        // Parse optional parameters EX, PX, NX, XX
        // 解析可选的 EX, PX, NX, XX 参数
        while let Ok(option) = parse.next_string() {
            match option.to_uppercase().as_str() {
                "EX" => {
                    // EX should be followed by a number (in seconds)
                    // EX 后面应跟数字（秒）
                    let exp = parse.next_string()?;
                    let exp_in_sec: u64 = exp.parse()?;
                    // Convert seconds to milliseconds
                    // 将秒转换为毫秒
                    expiration = Some(exp_in_sec * 1000);
                }
                "PX" => {
                    // PX should be followed by a number (in milliseconds)
                    // PX 后面应跟数字（毫秒）
                    let exp = parse.next_string()?;
                    let exp_in_ms: u64 = exp.parse()?;
                    // Use milliseconds directly
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
                _ => break, // Stop parsing when encountering an unknown option
                // 遇到未知的参数时，停止解析
            }
        }

        Ok(Set {
            key,
            value,
            expiration,
            nx,
            xx,
        })
    }
}
