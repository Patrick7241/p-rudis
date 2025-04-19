use std::io::Error;
use std::sync::{Arc, Mutex};
use crate::db::Db;
use crate::frame::Frame;
use crate::parse::Parse;

/// pubsub类型 publish命令
/// 向指定频道发布消息。如果频道不存在，会自动创建一个新频道。
/// 返回 `接收到消息的订阅者数量`，表示有多少个订阅者接收到了消息。
///
/// # 示例
///
/// ```text
/// PUBLISH channel message
/// ```
///
/// 如果有订阅者订阅该频道，则返回该频道接收到消息的订阅者数量。
pub struct Publish {
    channel: String,
    message: String,
}

impl Publish {
    /// 执行publish命令
    ///
    /// 根据解析的命令数据，从数据库获取相关频道，并向其发布消息
    /// 返回执行结果：`接收到消息的订阅者数量`
    pub fn publish_command(
        db: &mut Arc<Mutex<Db>>,
        parse: &mut Parse,
    ) -> crate::Result<Frame> {
        match Publish::parse_command(parse) {
            Ok(p) => {
                let mut db = db.lock().unwrap();
                // 向指定频道发送消息
                let received_count = db.publish(&p.channel, p.message);
                // 返回接收到消息的订阅者数量
                Ok(Frame::Integer(received_count as i64))
            }
            Err(_) => Ok(Frame::Error("ERR wrong number of arguments for 'publish' command".to_string())),
        }
    }

    /// 解析命令并返回一个Publish实例
    ///
    /// 解析出频道名称和消息内容，返回Publish结构体。
    fn parse_command(parse: &mut Parse) -> crate::Result<Self> {
        if parse.args_number()?!=2{
            return Err(Box::new(Error::new(std::io::ErrorKind::Other,"ERR wrong number of arguments for 'publish' command".to_string())));
        }
        // 解析频道名称
        let channel = parse.next_string()?;
        // 解析消息内容
        let message = parse.next_string()?;

        Ok(Publish { channel, message })
    }
}
