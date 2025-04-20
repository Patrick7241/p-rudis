use std::sync::{Arc, Mutex};
use crate::db::{Db, DbType};
use crate::frame::Frame;
use crate::parse::Parse;

/// Represents the `LRANGE` command in a Redis-like system.
///
/// The `LRANGE` command returns a range of elements from a list stored at the specified key.
/// The elements returned are from the list, starting at the specified start index and ending at the end index.
/// If the list is shorter than the requested range, it returns all available elements within the range.
///
/// The indices are 0-based, and negative indices can be used to specify positions from the end of the list:
/// - `-1` represents the last element of the list.
/// - `-2` represents the second-to-last element, and so on.
///
/// 表示 Redis 风格系统中的 `LRANGE` 命令。
///
/// `LRANGE` 命令返回指定键的列表中指定范围的元素。
/// 返回的元素是列表中从指定的起始索引到结束索引的元素。
/// 如果列表比请求的范围短，则返回范围内所有可用的元素。
///
/// 索引是从 0 开始的，负数索引可以用于指定从列表末尾开始的位置：
/// - `-1` 表示列表的最后一个元素。
/// - `-2` 表示倒数第二个元素，以此类推。
pub struct Lrange {
    key: String,  // The key of the list in the database. / 数据库中列表的键。
    start: isize, // The start index of the range. / 范围的起始索引。
    end: isize,   // The end index of the range. / 范围的结束索引。
}

impl Lrange {
    /// Executes the `LRANGE` command.
    ///
    /// This function processes the parsed command and returns a range of elements from the list at the given key.
    /// It handles the following scenarios:
    ///
    /// - If the key exists and contains a list, it returns the elements from the specified range.
    /// - If the key does not exist or is not a list, it returns an empty list.
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
    /// Returns a `Array` frame containing the elements in the specified range from the list.
    /// If the key does not exist or is not a list, it returns an empty array.
    ///
    /// 返回一个 `Array` 类型的帧，包含指定范围内的列表元素。
    /// 如果键不存在或不是列表类型，返回一个空数组。
    pub fn lrange_command(
        db: &mut Arc<Mutex<Db>>,
        parse: &mut Parse
    ) -> crate::Result<Frame> {
        match Lrange::parse_command(parse) {
            Ok(lrange) => {
                let mut db = db.lock().unwrap();
                match db.get_dbtype_mut(&lrange.key) {
                    // If the key exists and is a list, return the elements in the specified range.
                    // 如果键存在并且是列表类型，返回指定范围内的元素。
                    Some(DbType::List(list)) => {
                        let len = list.len() as isize;
                        // Adjust negative indices if necessary.
                        let start = if lrange.start < 0 {
                            len.saturating_add(lrange.start) // 防止负数下溢
                        } else {
                            lrange.start
                        }.clamp(0, len.saturating_sub(1)); // 确保在 0..len 范围内

                        let end = if lrange.end < 0 {
                            len.saturating_add(lrange.end)
                        } else {
                            lrange.end
                        }.clamp(0, len.saturating_sub(1)); // 确保在 0..len 范围内

                        let range = list
                            .iter()
                            .skip(start as usize)
                            .take((end - start + 1) as usize)
                            .map(|x| Frame::Bulk(x.clone().into_bytes()))
                            .collect::<Vec<Frame>>();

                        Ok(Frame::Array(range))
                    }
                    Some(_)=>{
                        Ok(Frame::Error("WRONGTYPE Operation against a key holding the wrong kind of value".to_string()))
                    }
                    // If the key does not exist, return an empty array.
                    // 如果键不存在，返回空数组。
                    _ => Ok(Frame::Array(Vec::new())),
                }
            }
            // If the command has an incorrect number of arguments, return an error.
            // 如果命令参数数量不正确，返回错误。
            Err(_) => {
                Ok(Frame::Error("ERR wrong number of arguments for 'lrange' command".to_string()))
            }
        }
    }

    /// Parses the `LRANGE` command, extracting the key and the range indices.
    ///
    /// This function expects the command to have at least three arguments: the key, start index, and end index.
    /// It returns the `Lrange` struct containing the parsed key, start, and end indices.
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing the `Lrange` struct with the parsed key, start index, and end index if successful.
    /// Otherwise, returns an error indicating that the number of arguments is incorrect.
    ///
    /// 解析 `LRANGE` 命令，提取键、起始索引和结束索引。
    ///
    /// 此函数期望命令至少有三个参数：键、起始索引和结束索引。
    /// 如果解析成功，返回包含解析后的键、起始索引和结束索引的 `Lrange` 结构体。
    /// 否则，返回一个错误，指示参数数量不正确。
    fn parse_command(parse: &mut Parse) -> crate::Result<Self> {
        // Check that there are exactly three arguments: the key, start index, and end index.
        // 检查命令有且仅有三个参数：键、起始索引和结束索引。
        if parse.args_number()? != 3 {
            return Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, "ERR wrong number of arguments for 'lrange' command")));
        }

        let key = parse.next_string()?;  // Parse the key. / 解析键。
        let start = parse.next_string()?; // Parse the start index. / 解析起始索引。
        let end = parse.next_string()?;   // Parse the end index. / 解析结束索引。

        if let (Ok(start), Ok(end)) = (start.parse::<isize>(), end.parse::<isize>()) {
            // If the start and end indices are valid, return the `Lrange` struct.
            // 如果起始索引和结束索引有效，返回 `Lrange` 结构体。
            Ok(Lrange {
                key,
                start,
                end,
            })
        }else{
            Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, "ERR invalid range indices for 'lrange' command")))
        }
    }
}
