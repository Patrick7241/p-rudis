use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use crate::db::{Db, DbType};
use crate::frame::Frame;
use crate::parse::Parse;
use crate::persistence::aof::propagate_aof;

/// Represents the `LREM` command in a Redis-like system.
///
/// The `LREM` command removes elements from a list that match the specified value.
/// The command allows you to remove elements from the head, tail, or from anywhere in the list depending on the count.
///
/// `LREM` 命令移除列表中与指定值匹配的元素。
/// 该命令允许您根据计数从列表的头部、尾部或列表中的任何位置移除元素。
pub struct Lrem {
    key: String,   // The key of the list in the database. / 数据库中列表的键。
    count: i64,    // The number of elements to remove. / 要移除的元素数量。
    value: String, // The value to remove from the list. / 要移除的值。
}

impl Lrem {
    /// Executes the `LREM` command.
    ///
    /// This function processes the parsed command and removes the specified number of elements that match the value from the list.
    /// It handles the following scenarios:
    ///
    /// - If the key exists and contains a list, it removes the specified number of matching elements.
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
    /// Returns the number of elements removed from the list.
    ///
    /// 返回从列表中移除的元素数量。
    pub fn lrem_command(
        db: &mut Arc<Mutex<Db>>,
        parse: &mut Parse
    ) -> crate::Result<Frame> {
        match Lrem::parse_command(parse) {
            Ok(lrem) => {
                let mut db = db.lock().unwrap();
                match db.get_dbtype_mut(&lrem.key) {
                    // If the key exists and is a list, remove the specified number of matching elements.
                    // 如果键存在且是列表类型，从列表中移除指定数量的匹配元素。
                    Some(DbType::List(list)) => {
                        let mut removed_count = 0;
                        if lrem.count == 0 {
                            // Remove all matching elements if count is 0.
                            // 如果 count 为 0，移除所有匹配的元素。
                            let len= list.len() as i64;
                            list.retain(|x| x != &lrem.value);
                            removed_count =len - list.len() as i64;
                        } else if lrem.count > 0 {
                            // Remove elements from the head of the list.
                            // 如果 count 大于 0，从列表的头部开始移除。
                            let mut i = 0;
                            while i < list.len() && removed_count < lrem.count {
                                if list[i] == lrem.value {
                                    list.remove(i);  // Directly remove the element.
                                    removed_count += 1;
                                } else {
                                    i += 1;
                                }
                            }
                        } else {
                            // Remove elements from the tail of the list.
                            // 如果 count 小于 0，从列表的尾部开始移除。
                            let mut i = list.len() as i64 - 1;
                            while i >= 0 && removed_count < -lrem.count {
                                if list[i as usize] == lrem.value {
                                    list.remove(i as usize);  // Directly remove the element.
                                    removed_count += 1;
                                }
                                i -= 1;
                            }
                        }
                        let args=vec![lrem.key,removed_count.to_string(),lrem.value];
                        propagate_aof("lrem".to_string(),args);
                        Ok(Frame::Integer(removed_count))
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
                Ok(Frame::Error("ERR wrong number of arguments for 'lrem' command".to_string()))
            }
        }
    }

    /// Parses the `LREM` command, extracting the key, count, and value.
    ///
    /// This function expects the command to have exactly three arguments: the key, count, and value.
    /// It returns the `Lrem` struct containing the parsed key, count, and value.
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing the `Lrem` struct with the parsed key, count, and value if successful.
    /// Otherwise, returns an error indicating that the number of arguments is incorrect.
    ///
    /// 解析 `LREM` 命令，提取键、计数和值。
    ///
    /// 此函数期望命令正好有三个参数：键、计数和值。
    /// 如果解析成功，返回包含解析后的键、计数和值的 `Lrem` 结构体。
    /// 否则，返回一个错误，指示参数数量不正确。
    fn parse_command(parse: &mut Parse) -> crate::Result<Self> {
        // Check that there are exactly three arguments: the key, count, and value.
        // 检查命令有且仅有三个参数：键、计数和值。
        if parse.args_number()? != 3 {
            return Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, "ERR wrong number of arguments for 'lrem' command")));
        }

        let key = parse.next_string()?; // Parse the key. / 解析键。
        let count = parse.next_string()?;  // Parse the count. / 解析计数。
        let value = parse.next_string()?; // Parse the value. / 解析值。

        let count = match count.parse::<i64>() {
            Ok(count) => count,
            Err(_) => {
                return Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, "ERR value is not an integer or out of range")));
            }
        };

        Ok(Lrem {
            key,
            count,
            value,
        })
    }
}
