use std::sync::{Arc, Mutex};
use crate::connection::ConnectionHandler;
use crate::db::Db;
use crate::frame::Frame;
use crate::parse::Parse;
use crate::shutdown::Shutdown;
use tokio::select;
use tokio::sync::broadcast;
use tokio_stream::{StreamExt, StreamMap};

/// pubsub 类型 `PUBLISH` 命令
/// 向指定频道发布消息。如果频道不存在，会自动创建一个新频道。
///
/// # 示例
///
/// ```text
/// PUBLISH channel message
/// ```

pub struct Subscribe {
    channel: String,
}

impl Subscribe {
    /// 执行 subscribe 命令
    pub async fn subscribe_command(
        db: &mut Arc<Mutex<Db>>,
        parse: &mut Parse,
        connection: &mut ConnectionHandler,
        shutdown: &mut Shutdown,
    ) -> crate::Result<()> {
        match Subscribe::parse_command(parse) {
            Ok(s) => {
                // 发送订阅确认消息
                let confirm_frames = vec![
                    Frame::Bulk("subscribe".into()),
                    Frame::Bulk(s.channel.clone().into()),
                    Frame::Integer(1), // TODO 订阅的频道数量，这里固定为1
                ];
                connection.write_data(Frame::Array(confirm_frames)).await?;
                let mut subscriptions = StreamMap::new();

                loop {
                    subscribe_to_channel(db,&s.channel, &mut subscriptions).await?;
                    select! {
                        Some((channel, msg)) = subscriptions.next() => {
                            println!("Received message from channel: {}", channel);
                             let msg=vec![
                               Frame::Bulk("message".into()),
                    Frame::Bulk(s.channel.clone().into()),
                    Frame::Bulk(msg.to_vec()), // TODO 订阅的频道数量，这里固定为1
                            ];
                            // 处理订阅消息
                            connection.write_data(Frame::Array(msg)).await?;
                        }
                        res=connection.read_data()=>{
                            let frame=match res?{
                                Some(frame) =>{
                                    // println!("Received frame: {:?}", frame);
                                     frame
                                }
                                // 接收订阅模式关闭信号
                                 None => return Ok(())
                            };
                            return Ok(());
                        }
                        _ = shutdown.recv() => {
                            // 处理关闭信号
                            println!("Received shutdown signal");
                            return Ok(());
                        }
                    }
                }
            }
            Err(_) => {
                // 如果解析失败，发送错误信息
                connection
                    .write_data(Frame::Error(
                        "ERR wrong number of arguments for 'subscribe' command".to_string(),
                    ))
                    .await?;
                Ok(())
            }
        }
    }

    /// 解析命令并返回一个 Subscribe 实例
    fn parse_command(parse: &mut Parse) -> crate::Result<Subscribe> {
        if parse.args_number()? != 1 {
            return Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                "ERR wrong number of arguments for 'subscribe' command".to_string(),
            )));
        }
        let channel = parse.next_string()?; // 解析频道名称

        Ok(Subscribe { channel })
    }
}


pub async fn subscribe_to_channel(db:&mut Arc<Mutex<Db>>, channel: &str, subscriptions: &mut StreamMap<String, crate::db::Messages>) -> crate::Result<()> {
    // 获取指定频道的可变引用并订阅该频道
    let mut guard = db.lock().unwrap();  // 先获取锁并绑定到变量
    let sender = guard.subscribe(channel); // 然后在同一作用域使用
    // println!("{:?}",sender);
    let mut receiver = sender.subscribe();

    // 使用 async_stream 创建一个异步流，用于接收频道的消息
    let receiver = Box::pin(async_stream::stream! {
        loop {
            match receiver.recv().await {
                Ok(msg) => yield msg,  // 成功接收到消息，进行转发
                Err(broadcast::error::RecvError::Lagged(_)) => {},  // 如果接收稍有延迟，跳过
                Err(_) => break,  // 如果发生其他错误，退出循环
            }
        }
    });

    // 将订阅的频道和对应的消息接收流插入到 subscriptions 中进行管理
    subscriptions.insert(channel.to_string(), receiver);

    // 返回成功
    Ok(())
}