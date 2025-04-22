use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use crate::db::{Db, DbType};
use crate::frame::Frame;
use crate::parse::Parse;
use std::time::{Duration, Instant};
use crate::persistence::aof::propagate_aof;

/// Represents the `BRPOP` command in a Redis-like system.
///
/// The `BRPOP` command is a blocking list pop operation that removes and returns the last element
/// of a list stored at the specified key. If the list is empty, the command will block until
/// either an element is available, or a timeout occurs.
///
/// `BRPOP` 命令是一个阻塞的列表弹出操作。它移除并返回指定键的列表中的最后一个元素。
/// 如果列表为空，命令会阻塞直到有元素可以弹出，或者超时。
pub struct Brpop {
    key: String,  // The key of the list in the database. / 数据库中列表的键。
    timeout: u64, // Timeout in seconds. / 超时时间（秒）。
}

impl Brpop {
    /// Executes the `BRPOP` command.
    ///
    /// This function processes the parsed command and performs the blocking pop operation.
    /// It handles the following scenarios:
    ///
    /// - If the list is non-empty, it pops the last element.
    /// - If the list is empty, it blocks until an element is available or the timeout is reached.
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
    /// Returns a `Frame` containing the popped value or an error if something goes wrong.
    ///
    /// 返回一个包含弹出值的 `Frame`，如果发生错误则返回错误。
    pub fn brpop_command(
        db: &mut Arc<Mutex<Db>>,
        parse: &mut Parse
    ) -> crate::Result<Frame> {
        match Brpop::parse_command(parse) {
            Ok(brpop) => {
                let mut db = db.lock().unwrap();
                db.set(&brpop.key, DbType::List(VecDeque::new()), None); // TODO: Simulate an empty list.
                match db.get_dbtype_mut(&brpop.key) {
                    Some(DbType::List(list)) => {
                        let start_time = Instant::now();
                        // Block until an element is available or the timeout is reached.
                        // 如果列表为空，阻塞直到有元素或者超时
                        while list.is_empty() {
                            if start_time.elapsed() >= Duration::new(brpop.timeout, 0) {
                                return Ok(Frame::Null); // Timeout reached.
                            }
                            // Simulate wait (this could be an actual sleep in a real system).
                            std::thread::sleep(Duration::from_millis(100)); // Check periodically.
                        }

                        // Pop the last element from the list.
                        let value = list.pop_back().unwrap();
                        propagate_aof("rpop".to_string(), vec![brpop.key.clone()]);
                        Ok(Frame::Bulk(value.into_bytes())) // Return the popped value.
                    },
                    // If the key exists but is not a list, return an error.
                    // 如果键存在但不是列表类型，返回错误。
                    Some(_) => {
                        Ok(Frame::Error("WRONGTYPE Operation against a key holding the wrong kind of value".to_string()))
                    },
                    // If the key does not exist, return nil.
                    // 如果键不存在，返回 nil。
                    None => {
                        Ok(Frame::Null)
                    }
                }
            },
            Err(_) => {
                // Incorrect number of arguments, return error.
                // 参数数量错误，返回错误。
                Ok(Frame::Error("ERR wrong number of arguments for 'brpop' command".to_string()))
            }
        }
    }

    /// Parses the `BRPOP` command, extracting the key and the timeout.
    ///
    /// This function expects the command to have at least two arguments: the key and the timeout.
    /// It returns the `Brpop` struct containing the parsed information.
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing the `Brpop` struct with the parsed key and timeout if successful.
    /// Otherwise, returns an error indicating that the number of arguments is incorrect.
    ///
    /// 返回一个 `Result`，如果解析成功，返回包含解析后的键和超时时间的 `Brpop` 结构体；否则，返回错误，指示参数数量不正确。
    fn parse_command(parse: &mut Parse) -> crate::Result<Self> {
        // The command requires exactly two arguments: the key and the timeout.
        // 命令需要正好两个参数：键和超时时间。
        if parse.args_number()? != 2 {
            return Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, "ERR wrong number of arguments for 'brpop' command")));
        }

        let key = parse.next_string()?; // Parse the key. / 解析键。
        let timeout = parse.next_string()?; // Parse the timeout. / 解析超时时间。

        let timeout = match timeout.parse::<u64>() {
            Ok(timeout) => timeout,
            Err(_) => {
                return Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, "ERR timeout is not a valid integer")));
            }
        };

        Ok(Brpop {
            key,
            timeout,
        })
    }
}
