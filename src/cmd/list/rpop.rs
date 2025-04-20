use std::sync::{Arc, Mutex};
use crate::db::{Db, DbType};
use crate::frame::Frame;
use crate::parse::Parse;

/// Represents the `RPOP` command in a Redis-like system.
///
/// The `RPOP` command removes and returns the last element (tail) from a list stored at the specified key.
/// If the list is empty, it returns `nil`. If the key does not exist, it returns `nil`.
///
/// 表示 Redis 风格系统中的 `RPOP` 命令。
///
/// `RPOP` 命令移除并返回指定键的列表的最后一个元素（尾部）。
/// 如果列表为空，返回 `nil`。如果键不存在，也返回 `nil`。
pub struct Rpop {
    key: String,   // The key of the list in the database. / 数据库中列表的键。
}

impl Rpop {
    /// Executes the `RPOP` command.
    ///
    /// This function processes the parsed command and removes the last element from the list at the given key.
    /// It handles the following scenarios:
    ///
    /// - If the key exists and contains a list, it removes the last element and returns it.
    /// - If the list is empty, it returns `nil`.
    /// - If the key does not exist or is not a list, it returns `nil`.
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
    /// Returns a `BulkString` frame with the value of the last element of the list.
    /// If the list is empty or the key does not exist, it returns `nil`.
    ///
    /// 返回一个 `BulkString` 类型的帧，包含列表的最后一个元素的值。
    /// 如果列表为空或键不存在，返回 `nil`。
    pub fn rpop_command(
        db: &mut Arc<Mutex<Db>>,
        parse: &mut Parse
    ) -> crate::Result<Frame> {
        match Rpop::parse_command(parse) {
            Ok(rpop) => {
                let mut db = db.lock().unwrap();
                match db.get_dbtype_mut(&rpop.key) {
                    // If the key exists and is a list, remove the last element and return it.
                    // 如果键存在并且是列表类型，移除列表的最后一个元素并返回它。
                    Some(DbType::List(list)) => {
                        if let Some(value) = list.pop_back() {
                            Ok(Frame::Bulk(value.into_bytes())) // Return the last element.
                        } else {
                            Ok(Frame::Null) // Return nil if the list is empty.
                        }
                    }
                    // If the key does not exist or is not a list, return nil.
                    // 如果键不存在或不是列表类型，返回 nil。
                    _ => Ok(Frame::Null),
                }
            }
            // If the command has an incorrect number of arguments, return an error.
            // 如果命令参数数量不正确，返回错误。
            Err(_) => {
                Ok(Frame::Error("ERR wrong number of arguments for 'rpop' command".to_string()))
            }
        }
    }

    /// Parses the `RPOP` command, extracting the key.
    ///
    /// This function expects the command to have exactly one argument: the key.
    /// It returns the `Rpop` struct containing the parsed key.
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing the `Rpop` struct with the parsed key if successful.
    /// Otherwise, returns an error indicating that the number of arguments is incorrect.
    ///
    /// 解析 `RPOP` 命令，提取键。
    ///
    /// 此函数期望命令恰好有一个参数：键。
    /// 如果解析成功，返回包含解析后的键的 `Rpop` 结构体。
    /// 否则，返回一个错误，指示参数数量不正确。
    fn parse_command(parse: &mut Parse) -> crate::Result<Self> {
        // Check that there is exactly one argument: the key.
        // 检查命令恰好有一个参数：键。
        if parse.args_number()? != 1 {
            return Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, "ERR wrong number of arguments for 'rpop' command")));
        }

        let key = parse.next_string()?; // Parse the key. / 解析键。

        Ok(Rpop {
            key,
        })
    }
}
