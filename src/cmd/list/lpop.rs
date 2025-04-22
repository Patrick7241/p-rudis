use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use libc::atexit;
use crate::db::{Db, DbType};
use crate::frame::Frame;
use crate::parse::Parse;
use crate::persistence::aof::propagate_aof;

/// Represents the `LPOP` command in a Redis-like system.
///
/// The `LPOP` command removes and returns the first element of the list stored at the specified key.
/// If the key does not exist or the list is empty, the command returns `nil` (null).
/// The command returns the value of the element removed from the list.
///
/// 表示 Redis 风格系统中的 `LPOP` 命令。
///
/// `LPOP` 命令删除并返回存储在指定键的列表中的第一个元素。
/// 如果键不存在或列表为空，命令返回 `nil`（空值）。
/// 命令返回从列表中移除的元素的值。
pub struct Lpop {
    key: String,   // The key of the list in the database. / 数据库中列表的键。
}

impl Lpop {
    /// Executes the `LPOP` command.
    ///
    /// This function processes the parsed command and removes the first element from the list at the given key.
    /// It handles the following scenarios:
    ///
    /// - If the key exists and contains a list, it removes the first element and returns it.
    /// - If the key does not exist or the list is empty, it returns `nil`.
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
    /// Returns a `String` frame with the value of the element removed, or `nil` if the list is empty or does not exist.
    ///
    /// 返回一个 `String` 类型的帧，包含被移除的元素的值，如果列表为空或键不存在，则返回 `nil`。
    pub fn lpop_command(
        db: &mut Arc<Mutex<Db>>,
        parse: &mut Parse
    ) -> crate::Result<Frame> {
        match Lpop::parse_command(parse) {
            Ok(lpop) => {
                let mut db = db.lock().unwrap();
                match db.get_dbtype_mut(&lpop.key) {
                    // If the key exists and is a list, remove and return the first element.
                    // 如果键存在并且是列表类型，删除并返回第一个元素。
                    Some(DbType::List(list)) => {
                        if let Some(value) = list.pop_front() {
                            let ars = vec![lpop.key.clone()];
                            propagate_aof("lpop".to_string(), ars);
                            Ok(Frame::Bulk(value.into_bytes()))
                        } else {
                            // If the list is empty, return nil.
                            // 如果列表为空，返回 nil。
                            Ok(Frame::Null)
                        }
                    }
                    // If the key exists but is not a list, return an error.
                    // 如果键存在，但不是列表类型，返回错误。
                    Some(_) => {
                        Ok(Frame::Error("WRONGTYPE Operation against a key holding the wrong kind of value".to_string()))
                    }
                    // If the key does not exist, return nil.
                    // 如果键不存在，返回 nil。
                    None => {
                        Ok(Frame::Null)
                    }
                }
            }
            // If the command has an incorrect number of arguments, return an error.
            // 如果命令参数数量不正确，返回错误。
            Err(_) => {
                Ok(Frame::Error("ERR wrong number of arguments for 'lpop' command".to_string()))
            }
        }
    }

    /// Parses the `LPOP` command, extracting the key.
    ///
    /// This function expects the command to have exactly one argument: the key.
    /// It returns the `Lpop` struct containing the parsed key.
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing the `Lpop` struct with the parsed key if successful.
    /// Otherwise, returns an error indicating that the number of arguments is incorrect.
    ///
    /// 返回一个 `Result`，如果解析成功，返回包含解析后的键的 `Lpop` 结构体。如果失败，返回错误，指示参数数量不正确。
    fn parse_command(parse: &mut Parse) -> crate::Result<Self> {
        // The command requires exactly one argument: the key.
        // 命令需要正好一个参数：键。
        if parse.args_number()? != 1 {
            return Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, "ERR wrong number of arguments for 'lpop' command")));
        }

        let key = parse.next_string()?; // Parse the key. / 解析键。

        Ok(Lpop {
            key,
        })
    }
}
