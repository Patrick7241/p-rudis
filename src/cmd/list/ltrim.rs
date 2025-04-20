use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use crate::db::{Db, DbType};
use crate::frame::Frame;
use crate::parse::Parse;

/// Represents the `LTRIM` command in a Redis-like system.
///
/// The `LTRIM` command trims a list to only contain the elements in the specified range.
/// If the range is out of bounds, the list will be truncated accordingly.
///
/// `LTRIM` 命令将列表修剪为仅包含指定范围内的元素。
/// 如果范围超出边界，列表将被相应地截断。
pub struct Ltrim {
    key: String,  // The key of the list in the database. / 数据库中列表的键。
    start: i64,   // The start index for the range. / 范围的起始索引。
    stop: i64,    // The stop index for the range. / 范围的结束索引。
}

impl Ltrim {
    /// Executes the `LTRIM` command.
    ///
    /// This function processes the parsed command and trims the list to only contain elements in the specified range.
    /// It handles the following scenarios:
    ///
    /// - If the key exists and is a list, it trims the list to the specified range.
    /// - If the key does not exist or is not a list, it returns an error.
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
    /// Returns an `Integer` frame with the length of the list after trimming.
    ///
    /// 返回一个 `Integer` 类型的帧，表示修剪后的列表长度。
    pub fn ltrim_command(
        db: &mut Arc<Mutex<Db>>,
        parse: &mut Parse
    ) -> crate::Result<Frame> {
        match Ltrim::parse_command(parse) {
            Ok(ltrim) => {
                let mut db = db.lock().unwrap();
                match db.get_dbtype_mut(&ltrim.key) {
                    // If the key exists and is a list, trim the list to the specified range.
                    // 如果键存在并且是列表类型，修剪列表为指定范围。
                    Some(DbType::List(list)) => {
                        // Ensure the range is within bounds.
                        let len = list.len() as i64;
                        let start = if ltrim.start < 0 {
                            len.saturating_add(ltrim.start) // 防止负数下溢
                        } else {
                            ltrim.start
                        }.clamp(0, len.saturating_sub(1)); // 确保 start ∈ [0, len-1]

                        let stop = if ltrim.stop < 0 {
                            len.saturating_add(ltrim.stop) // 防止负数下溢
                        } else {
                            ltrim.stop
                        }.clamp(0, len.saturating_sub(1)); // 确保 stop ∈ [0, len-1]

                        if start <= stop {
                            let trimmed: VecDeque<String> = list.iter().skip(start as usize).take((stop - start + 1) as usize).cloned().collect();
                            *list = trimmed;
                        } else {
                            // If the start index is greater than the stop index, the list will be empty.
                            // 如果起始索引大于结束索引，列表将为空。
                            list.clear();
                        }

                        Ok(Frame::Integer(list.len() as i64))
                    }
                    // If the key does not exist or is not a list, return an error.
                    // 如果键不存在或不是列表类型，返回错误。
                    _ => {
                        Ok(Frame::Error("WRONGTYPE Operation against a key holding the wrong kind of value".to_string()))
                    }
                }
            }
            // If the command has an incorrect number of arguments, return an error.
            // 如果命令参数数量不正确，返回错误。
            Err(_) => {
                Ok(Frame::Error("ERR wrong number of arguments for 'ltrim' command".to_string()))
            }
        }
    }

    /// Parses the `LTRIM` command, extracting the key, start index, and stop index.
    ///
    /// This function expects the command to have exactly three arguments: the key, start, and stop indices.
    /// It returns the `Ltrim` struct containing the parsed key, start, and stop indices.
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing the `Ltrim` struct with the parsed key, start, and stop indices if successful.
    /// Otherwise, returns an error indicating that the number of arguments is incorrect.
    ///
    /// 解析 `LTRIM` 命令，提取键、起始索引和结束索引。
    ///
    /// 此函数期望命令正好有三个参数：键、起始索引和结束索引。
    /// 如果解析成功，返回包含解析后的键、起始索引和结束索引的 `Ltrim` 结构体。
    /// 否则，返回一个错误，指示参数数量不正确。
    fn parse_command(parse: &mut Parse) -> crate::Result<Self> {
        // Check that there are exactly three arguments: the key, start, and stop indices.
        // 检查命令有且仅有三个参数：键、起始索引和结束索引。
        if parse.args_number()? != 3 {
            return Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, "ERR wrong number of arguments for 'ltrim' command")));
        }

        let key = parse.next_string()?;  // Parse the key. / 解析键。
        let start = parse.next_string()?;   // Parse the start index. / 解析起始索引。
        let stop = parse.next_string()?;    // Parse the stop index. / 解析结束索引。

        let start = match start.parse::<i64>() {
            Ok(num) => num,
            Err(_) => return Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, "ERR value is not an integer or out of range"))),
        };
        let stop = match stop.parse::<i64>() {
            Ok(num) => num,
            Err(_) => return Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, "ERR value is not an integer or out of range"))),
        };

        Ok(Ltrim {
            key,
            start,
            stop,
        })
    }
}
