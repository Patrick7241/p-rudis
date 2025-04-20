use std::sync::{Arc, Mutex};
use crate::db::{Db, DbType};
use crate::frame::Frame;
use crate::parse::Parse;

/// Represents the `LLEN` command in a Redis-like system.
///
/// The `LLEN` command returns the length of the list stored at the specified key.
/// If the key does not exist or is not a list, it returns `0`.
///
/// 表示 Redis 风格系统中的 `LLEN` 命令。
///
/// `LLEN` 命令返回指定键的列表的长度。如果键不存在或不是列表类型，则返回 `0`。
pub struct Llen {
    key: String,  // The key of the list in the database. / 数据库中列表的键。
}

impl Llen {
    /// Executes the `LLEN` command.
    ///
    /// This function processes the parsed command and returns the length of the list at the given key.
    /// If the key does not exist or is not a list, it returns `0`.
    ///
    /// # Arguments
    ///
    /// - `db`: A mutable reference to the database (`Arc<Mutex<Db>>`), where the list is stored.
    ///         / 数据库 (`Arc<Mutex<Db>>`) 的可变引用，存储列表的位置。
    /// - `parse`: A reference to the parser that contains the parsed command.
    ///            / 解析器的引用，包含解析后的命令。
    ///
    /// # Returns
    ///
    /// Returns an `Integer` frame with the length of the list. If the key does not exist or is not a list, returns `0`.
    ///
    /// 返回一个 `Integer` 类型的帧，表示列表的长度。如果键不存在或不是列表类型，返回 `0`。
    pub fn llen_command(
        db: &mut Arc<Mutex<Db>>,
        parse: &mut Parse
    ) -> crate::Result<Frame> {
        match Llen::parse_command(parse) {
            Ok(llen) => {
                let mut db = db.lock().unwrap();
                match db.get_dbtype_mut(&llen.key) {
                    // If the key exists and is a list, return the length of the list.
                    // 如果键存在并且是列表类型，返回列表的长度。
                    Some(DbType::List(list)) => {
                        Ok(Frame::Integer(list.len() as i64))
                    }
                    // If the key does not exist or is not a list, return 0.
                    // 如果键不存在或不是列表类型，返回 0。
                    _ => Ok(Frame::Integer(0)),
                }
            }
            // If the command has an incorrect number of arguments, return an error.
            // 如果命令参数数量不正确，返回错误。
            Err(_) => {
                Ok(Frame::Error("ERR wrong number of arguments for 'llen' command".to_string()))
            }
        }
    }

    /// Parses the `LLEN` command, extracting the key.
    ///
    /// This function expects the command to have exactly one argument: the key.
    /// It returns the `Llen` struct containing the parsed key.
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing the `Llen` struct with the parsed key if successful.
    /// Otherwise, returns an error indicating that the number of arguments is incorrect.
    ///
    /// 解析 `LLEN` 命令，提取键。
    ///
    /// 此函数期望命令正好有一个参数：键。
    /// 如果解析成功，返回包含解析后的键的 `Llen` 结构体。
    /// 否则，返回一个错误，指示参数数量不正确。
    fn parse_command(parse: &mut Parse) -> crate::Result<Self> {
        // Check that there is exactly one argument: the key.
        // 检查命令有且仅有一个参数：键。
        if parse.args_number()? != 1 {
            return Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, "ERR wrong number of arguments for 'llen' command")));
        }

        let key = parse.next_string()?; // Parse the key. / 解析键。

        Ok(Llen {
            key,
        })
    }
}
