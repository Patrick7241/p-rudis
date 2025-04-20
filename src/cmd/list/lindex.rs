use std::sync::{Arc, Mutex};
use crate::db::{Db, DbType};
use crate::frame::Frame;
use crate::parse::Parse;

/// Represents the `LINDEX` command in a Redis-like system.
///
/// The `LINDEX` command returns the element at the specified index in the list stored at the given key.
/// If the index is out of range, it returns a `nil` frame.
///
/// The indices are 0-based, and negative indices can be used to specify positions from the end of the list:
/// - `-1` represents the last element of the list.
/// - `-2` represents the second-to-last element, and so on.
///
/// 表示 Redis 风格系统中的 `LINDEX` 命令。
///
/// `LINDEX` 命令返回指定键的列表中指定索引位置的元素。
/// 如果索引超出范围，则返回 `nil` 帧。
///
/// 索引是从 0 开始的，负数索引可以用于指定从列表末尾开始的位置：
/// - `-1` 表示列表的最后一个元素。
/// - `-2` 表示倒数第二个元素，以此类推。
pub struct Lindex {
    key: String,  // The key of the list in the database. / 数据库中列表的键。
    index: isize, // The index of the element to retrieve. / 要获取的元素的索引。
}

impl Lindex {
    /// Executes the `LINDEX` command.
    ///
    /// This function processes the parsed command and returns the element at the specified index in the list at the given key.
    /// If the index is out of range, it returns a `nil` frame.
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
    /// Returns a `Bulk` frame with the element at the specified index, or `nil` if the index is out of range.
    ///
    /// 返回一个 `Bulk` 类型的帧，包含指定索引位置的元素。如果索引超出范围，则返回 `nil`。
    pub fn lindex_command(
        db: &mut Arc<Mutex<Db>>,
        parse: &mut Parse
    ) -> crate::Result<Frame> {
        match Lindex::parse_command(parse) {
            Ok(lindex) => {
                let mut db = db.lock().unwrap();
                match db.get_dbtype_mut(&lindex.key) {
                    // If the key exists and is a list, return the element at the specified index.
                    // 如果键存在并且是列表类型，返回指定索引位置的元素。
                    Some(DbType::List(list)) => {
                        let len = list.len() as isize;
                        let index = if lindex.index < 0 {
                            len.saturating_add(lindex.index)
                        } else {
                            lindex.index
                        }.clamp(0, len.saturating_sub(1));

                        // Check if the index is within bounds.
                        // 检查索引是否在有效范围内。
                        if index >= 0 && index < len {
                            Ok(Frame::Bulk(list[index as usize].clone().into_bytes()))
                        } else {
                            // If the index is out of range, return `nil`.
                            // 如果索引超出范围，返回 `nil`。
                            Ok(Frame::Null)
                        }
                    }
                    // If the key does not exist or is not a list, return `nil`.
                    // 如果键不存在或不是列表类型，返回 `nil`。
                    _ => Ok(Frame::Null),
                }
            }
            // If the command has an incorrect number of arguments, return an error.
            // 如果命令参数数量不正确，返回错误。
            Err(_) => {
                Ok(Frame::Error("ERR wrong number of arguments for 'lindex' command".to_string()))
            }
        }
    }

    /// Parses the `LINDEX` command, extracting the key and index.
    ///
    /// This function expects the command to have exactly two arguments: the key and the index.
    /// It returns the `Lindex` struct containing the parsed key and index.
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing the `Lindex` struct with the parsed key and index if successful.
    /// Otherwise, returns an error indicating that the number of arguments is incorrect.
    ///
    /// 解析 `LINDEX` 命令，提取键和索引。
    ///
    /// 此函数期望命令正好有两个参数：键和索引。
    /// 如果解析成功，返回包含解析后的键和索引的 `Lindex` 结构体。
    /// 否则，返回一个错误，指示参数数量不正确。
    fn parse_command(parse: &mut Parse) -> crate::Result<Self> {
        // Check that there are exactly two arguments: the key and the index.
        // 检查命令有且仅有两个参数：键和索引。
        if parse.args_number()? != 2 {
            return Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, "ERR wrong number of arguments for 'lindex' command")));
        }

        let key = parse.next_string()?;    // Parse the key. / 解析键。
        let index = parse.next_string()?;   // Parse the index. / 解析索引。

        let index = match index.parse::<isize>() {
            Ok(index) => index,
            Err(_) => {
                return Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, "ERR value is not an integer or out of range")));
            }
        };

        Ok(Lindex {
            key,
            index,
        })
    }
}
