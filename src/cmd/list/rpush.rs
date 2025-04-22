use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use crate::db::{Db, DbType};
use crate::frame::Frame;
use crate::parse::Parse;
use crate::persistence::aof::propagate_aof;

/// Represents the `RPUSH` command in a Redis-like system.
///
/// The `RPUSH` command inserts one or more elements at the tail (right) of a list stored at the specified key.
/// If the key does not exist, it creates a new list and inserts the elements at the tail.
/// The command returns the length of the list after the operation.
///
/// 表示 Redis 风格系统中的 `RPUSH` 命令。
///
/// `RPUSH` 命令将一个或多个元素插入到指定键的列表的尾部（右侧）。
/// 如果键不存在，它会创建一个新的列表并将元素插入尾部。命令返回操作后的列表长度。
pub struct Rpush {
    key: String,   // The key of the list in the database. / 数据库中列表的键。
    values: Vec<String>, // The values to insert into the list. / 要插入列表的值。
}

impl Rpush {
    /// Executes the `RPUSH` command.
    ///
    /// This function processes the parsed command and inserts the specified elements into the list at the given key.
    /// It handles the following scenarios:
    ///
    /// - If the key exists and contains a list, it inserts the specified elements at the tail of the list.
    /// - If the key does not exist, it creates a new list and inserts the elements at the tail.
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
    /// Returns an `Integer` frame with the length of the list after the operation.
    ///
    /// 返回一个 `Integer` 类型的帧，表示操作后的列表长度。
    pub fn rpush_command(
        db: &mut Arc<Mutex<Db>>,
        parse: &mut Parse
    ) -> crate::Result<Frame> {
        match Rpush::parse_command(parse) {
            Ok(rpush) => {
                let mut db = db.lock().unwrap();
                match db.get_dbtype_mut(&rpush.key) {
                    // If the key exists and is a list, insert the values at the tail of the list.
                    // 如果键存在并且是列表类型，将值插入列表尾部。
                    Some(DbType::List(list)) => {
                        let mut args= vec![rpush.key.to_string()];
                        for value in rpush.values.iter() {
                            args.push(value.to_string());
                            list.push_back(value.to_string());
                        }
                        propagate_aof("rpush".to_string(), args);
                        Ok(Frame::Integer(list.len() as i64))
                    }
                    // If the key exists but is not a list, return an error.
                    // 如果键存在，但不是列表类型，返回错误。
                    Some(_) => {
                        Ok(Frame::Error("WRONGTYPE Operation against a key holding the wrong kind of value".to_string()))
                    }
                    // If the key does not exist, create a new list and insert the values.
                    // 如果键不存在，创建一个新的列表并插入值。
                    None => {
                        let mut list = VecDeque::new();
                        let mut args = vec![rpush.key.to_string()];
                        for value in rpush.values.iter() {
                            args.push(value.to_string());
                            list.push_back(value.to_string());
                        }
                        propagate_aof("rpush".to_string(), args);
                        let len = list.len();
                        db.set(rpush.key.as_str(), DbType::List(list), None);
                        Ok(Frame::Integer(len as i64))
                    }
                }
            }
            // If the command has an incorrect number of arguments, return an error.
            // 如果命令参数数量不正确，返回错误。
            Err(_) => {
                Ok(Frame::Error("ERR wrong number of arguments for 'rpush' command".to_string()))
            }
        }
    }

    /// Parses the `RPUSH` command, extracting the key and the list of values.
    ///
    /// This function expects the command to have at least two arguments: the key and at least one value.
    /// It returns the `Rpush` struct containing the parsed information.
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing the `Rpush` struct with the parsed key and values if successful.
    /// Otherwise, returns an error indicating that the number of arguments is incorrect.
    ///
    /// 解析 `RPUSH` 命令，提取键和一系列值。
    ///
    /// 此函数期望命令至少有两个参数：键和至少一个值。
    /// 如果解析成功，返回包含解析后的键和值的 `Rpush` 结构体。
    /// 否则，返回一个错误，指示参数数量不正确。
    fn parse_command(parse: &mut Parse) -> crate::Result<Self> {
        // Check that there are at least two arguments: the key and at least one value.
        // 检查命令至少有两个参数：键和至少一个值。
        if parse.args_number()? < 2 {
            return Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, "ERR wrong number of arguments for 'rpush' command")));
        }

        let key = parse.next_string()?; // Parse the key. / 解析键。
        let mut values = Vec::new();

        // Parse the values to be inserted into the list.
        // 解析要插入列表的多个值。
        while let Ok(value) = parse.next_string() {
            values.push(value);
        }

        Ok(Rpush {
            key,
            values,
        })
    }
}
