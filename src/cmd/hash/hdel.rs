use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use crate::db::{Db, DbType};
use crate::frame::Frame;
use crate::parse::Parse;
use crate::persistence::aof::propagate_aof;

/// Represents the `HDEL` command in a Redis-like system.
/// `HDEL` 命令用于删除哈希表中指定的字段。
pub struct Hdel {
    key: String,               // The key of the hash in the database. / 数据库中哈希表的键。
    fields: Vec<String>,       // A list of field names to remove from the hash. / 要从哈希表中删除的字段名称列表。
}

impl Hdel {
    /// Executes the `HDEL` command.
    /// 执行 `HDEL` 命令。
    ///
    /// # Arguments
    /// - `db`: A mutable reference to the database (`Arc<Mutex<Db>>`), where the hash is stored.
    ///         / 数据库 (`Arc<Mutex<Db>>`) 的可变引用，存储哈希表的位置。
    /// - `parse`: A reference to the parser that contains the parsed command.
    ///            / 解析器的引用，包含解析后的命令。
    ///
    /// # Returns
    /// Returns the number of fields removed as an `i64` integer wrapped in a `Frame` if successful.
    /// Returns an error frame if the command arguments are invalid or the operation fails.
    /// 如果成功，返回删除字段数量的 `i64` 类型整数封装在 `Frame` 中。
    /// 如果命令参数无效或操作失败，则返回错误帧。
    pub fn hdel_command(
        db: &mut Arc<Mutex<Db>>,
        parse: &mut Parse
    ) -> crate::Result<Frame> {
        match Hdel::parse_command(parse) {
            Ok(hdel) => {
                let mut db = db.lock().unwrap();
                match db.get_dbtype_mut(&hdel.key) {
                    Some(DbType::Hash(hash)) => {
                        // Count the number of deleted fields / 统计删除的字段数量
                        let mut deleted_count = 0;

                        // Delete fields and count the deletions / 删除字段并计数
                        for field in &hdel.fields {
                            if hash.remove(field).is_some() {
                                deleted_count += 1;
                                // Propagate the delete operation to AOF for each deleted field
                                // 每删除一个字段，就将该删除操作传播到 AOF
                                Hdel::propagate_aof("hdel", &hdel.key, field);
                            }
                        }

                        // Return the number of deleted fields / 返回删除字段的数量
                        Ok(Frame::Integer(deleted_count as i64))
                    },
                    Some(_) => {
                        // Key exists, but type mismatch, return WRONGTYPE error / 键存在，但类型不匹配，返回 WRONGTYPE 错误
                        Ok(Frame::Error("WRONGTYPE Operation against a key holding the wrong kind of value".to_string()))
                    },
                    None => {
                        // Key does not exist, return 0 / 键不存在，返回 0
                        Ok(Frame::Integer(0))
                    }
                }
            }
            Err(_) => {
                // Incorrect number of arguments, return error / 参数数量错误，返回错误
                Ok(Frame::Error("ERR wrong number of arguments for 'hdel' command".to_string()))
            }
        }
    }

    /// Parses the command and extracts the fields to be deleted from the hash.
    /// 解析命令并提取要删除的哈希表字段。
    ///
    /// # Returns
    /// Returns a `Result` containing the `Hdel` struct with the parsed key and fields if successful.
    /// Otherwise, returns an error frame indicating the problem.
    /// 如果成功，返回包含解析后的键和字段的 `Hdel` 结构体。如果失败，返回错误帧以指示问题。
    fn parse_command(parse: &mut Parse) -> crate::Result<Self> {
        // The minimum number of arguments required is 2: key and at least one field.
        // 所需的最少参数为 2：键和至少一个字段。
        let args_number = parse.args_number()?;
        if args_number < 2 {
            return Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, "ERR wrong number of arguments for 'hdel' command")));
        }

        let key = parse.next_string()?;
        let mut fields = Vec::with_capacity(args_number - 1);

        // Collect all fields / 获取所有字段
        for _ in 0..(args_number - 1) {
            let field = parse.next_string()?;
            fields.push(field);
        }

        Ok(Hdel { key, fields })
    }

    /// Propagates the `HDEL` command to AOF.
    /// 将 `HDEL` 命令传播到 AOF。
    fn propagate_aof(command: &str, key: &str, field: &str) {
        // Propagate the HDEL command with key and field to AOF.
        // 将 `HDEL` 命令与键和字段传播到 AOF。
        let args = vec![key.to_string(), field.to_string()];
        propagate_aof(command.to_string(), args);
    }
}
