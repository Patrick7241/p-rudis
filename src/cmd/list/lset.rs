use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use crate::db::{Db, DbType};
use crate::frame::Frame;
use crate::parse::Parse;

/// Represents the `LSET` command in a Redis-like system.
///
/// The `LSET` command sets the value of an element in a list at the specified index.
/// If the index is out of range, it returns an error.
///
/// 表示 Redis 风格系统中的 `LSET` 命令。
///
/// `LSET` 命令在指定索引处设置列表中元素的值。如果索引超出范围，返回错误。
pub struct Lset {
    key: String,   // The key of the list in the database. / 数据库中列表的键。
    index: i64,    // The index of the element to set. / 要设置的元素的索引。
    value: String, // The value to set at the specified index. / 要设置的值。
}

impl Lset {
    /// Executes the `LSET` command.
    ///
    /// This function processes the parsed command and sets the value of the element at the specified index.
    /// It handles the following scenarios:
    ///
    /// - If the key exists and contains a list, it sets the value at the specified index.
    /// - If the index is out of range, it returns an error.
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
    /// Returns an `OK` frame if the operation was successful.
    /// If the index is out of range, returns an error frame.
    ///
    /// 返回 `OK` 类型的帧，如果操作成功。 如果索引超出范围，返回错误帧。
    pub fn lset_command(
        db: &mut Arc<Mutex<Db>>,
        parse: &mut Parse
    ) -> crate::Result<Frame> {
        match Lset::parse_command(parse) {
            Ok(lset) => {
                let mut db = db.lock().unwrap();
                match db.get_dbtype_mut(&lset.key) {
                    // If the key exists and is a list, set the value at the specified index.
                    // 如果键存在并且是列表类型，在指定的索引位置设置值。
                    Some(DbType::List(list)) => {
                        if lset.index < 0 || lset.index >= list.len() as i64 {
                            // If the index is out of range, return an error.
                            // 如果索引超出范围，返回错误。
                            Ok(Frame::Error("ERR index out of range".to_string()))
                        } else {
                            list[lset.index as usize] = lset.value;
                            Ok(Frame::Simple("OK".to_string()))
                        }
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
                Ok(Frame::Error("ERR wrong number of arguments for 'lset' command".to_string()))
            }
        }
    }

    /// Parses the `LSET` command, extracting the key, index, and value.
    ///
    /// This function expects the command to have exactly three arguments: the key, index, and value.
    /// It returns the `Lset` struct containing the parsed key, index, and value.
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing the `Lset` struct with the parsed key, index, and value if successful.
    /// Otherwise, returns an error indicating that the number of arguments is incorrect.
    ///
    /// 解析 `LSET` 命令，提取键、索引和值。
    ///
    /// 此函数期望命令正好有三个参数：键、索引和值。
    /// 如果解析成功，返回包含解析后的键、索引和值的 `Lset` 结构体。
    /// 否则，返回一个错误，指示参数数量不正确。
    fn parse_command(parse: &mut Parse) -> crate::Result<Self> {
        // Check that there are exactly three arguments: the key, index, and value.
        // 检查命令有且仅有三个参数：键、索引和值。
        if parse.args_number()? != 3 {
            return Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, "ERR wrong number of arguments for 'lset' command")));
        }

        let key = parse.next_string()?; // Parse the key. / 解析键。
        let index = parse.next_string()?;  // Parse the index. / 解析索引。
        let value = parse.next_string()?; // Parse the value. / 解析值。

        let index = match index.parse::<i64>() {
            Ok(index) => index,
            Err(_) => {
                return Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, "ERR value is not an integer or out of range")));
            }
        };

        Ok(Lset {
            key,
            index,
            value,
        })
    }
}
