use std::io::Error;
use std::sync::{Arc, Mutex};
use crate::db::Db;
use crate::frame::Frame;
use crate::parse::Parse;

/// Represents the `PUBLISH` command for a Redis-like pub/sub system.
///
/// The `PUBLISH` command sends a message to a specified channel. If the channel does not exist,
/// it will be automatically created. The command returns the number of subscribers that received the message.
///
/// `PUBLISH` 命令向指定频道发送消息。如果频道不存在，会自动创建一个新频道。返回接收到消息的订阅者数量。
///
/// # Example
///
/// ```text
/// PUBLISH channel message
/// ```
///
/// If there are subscribers to the channel, it returns the number of subscribers that received the message.
/// 如果有订阅者订阅该频道，则返回该频道接收到消息的订阅者数量。
pub struct Publish {
    channel: String,  // The channel name to which the message will be published. / 发布消息的频道名称
    message: String,  // The message content to be sent. / 要发送的消息内容
}

impl Publish {
    /// Executes the `PUBLISH` command.
    ///
    /// This function processes the parsed `PUBLISH` command and sends the message to the specified channel.
    /// It returns the number of subscribers that received the message.
    ///
    /// # Arguments
    ///
    /// - `db`: A mutable reference to the database (`Arc<Mutex<Db>>`), where the channels and subscriptions are managed.
    ///         / 数据库 (`Arc<Mutex<Db>>`) 的可变引用，管理频道和订阅。
    /// - `parse`: A reference to the parser that contains the parsed command.
    ///            / 解析器的引用，包含解析后的命令。
    ///
    /// # Returns
    ///
    /// Returns an `Integer` frame with the number of subscribers that received the message.
    ///
    /// 返回一个 `Integer` 类型的帧，表示接收到消息的订阅者数量。
    pub fn publish_command(
        db: &mut Arc<Mutex<Db>>,
        parse: &mut Parse,
    ) -> crate::Result<Frame> {
        match Publish::parse_command(parse) {
            Ok(p) => {
                let mut db = db.lock().unwrap();
                // Publish the message to the specified channel
                // 向指定频道发布消息
                let received_count = db.publish(&p.channel, p.message);
                // Return the number of subscribers who received the message
                // 返回接收到消息的订阅者数量
                Ok(Frame::Integer(received_count as i64))
            }
            Err(_) => Ok(Frame::Error("ERR wrong number of arguments for 'publish' command".to_string())),
        }
    }

    /// Parses the `PUBLISH` command, extracting the channel name and message content.
    ///
    /// # Returns
    ///
    /// Returns a `Publish` instance containing the parsed channel and message.
    ///
    /// 返回一个包含解析后的频道名称和消息内容的 `Publish` 结构体实例。
    fn parse_command(parse: &mut Parse) -> crate::Result<Self> {
        if parse.args_number()? != 2 {
            return Err(Box::new(Error::new(std::io::ErrorKind::Other, "ERR wrong number of arguments for 'publish' command".to_string())));
        }
        // Parse the channel name
        // 解析频道名称
        let channel = parse.next_string()?;
        // Parse the message content
        // 解析消息内容
        let message = parse.next_string()?;

        Ok(Publish { channel, message })
    }
}
