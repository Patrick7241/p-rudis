use std::sync::{Arc, Mutex};
use crate::connection::ConnectionHandler;
use crate::db::Db;
use crate::frame::Frame;
use crate::parse::Parse;
use crate::shutdown::Shutdown;
use tokio::select;
use tokio::sync::broadcast;
use tokio_stream::{StreamExt, StreamMap};

/// `Subscribe` 结构体用于表示订阅操作
///
/// 该结构体包含一个频道列表，表示客户端希望订阅的所有频道。
///
/// # 示例
///
/// ```text
/// SUBSCRIBE channel1 channel2 ...
/// ```
pub struct Subscribe {
    channels: Vec<String>,
}

impl Subscribe {
    /// 执行 `subscribe` 命令
    ///
    /// 该方法将解析传入的命令并进行相应的订阅操作。
    /// 它将为每个频道发送确认消息，并且在后台订阅这些频道以接收消息。
    ///
    /// # 参数
    /// - `db`: 用于访问数据库的共享引用。
    /// - `parse`: 用于解析客户端传来的命令。
    /// - `connection`: 用于与客户端进行数据通信的连接句柄。
    /// - `shutdown`: 用于接收关闭信号的通道。
    ///
    /// # 返回
    /// 如果命令解析成功，则返回订阅操作的结果。
    /// 如果命令解析失败，则会发送错误消息并退出。
    pub async fn subscribe_command(
        db: &mut Arc<Mutex<Db>>,
        parse: &mut Parse,
        connection: &mut ConnectionHandler,
        shutdown: &mut Shutdown,
    ) -> crate::Result<()> {
        match Subscribe::parse_command(parse) {
            Ok(s) => {
                // 发送订阅确认消息，类似于 Redis 的行为
                for (index, channel) in s.channels.iter().enumerate() {
                    let confirm_frames = vec![
                        Frame::Bulk("subscribe".into()),  // 表示 subscribe 命令
                        Frame::Bulk(channel.clone().into()),  // 当前频道的名称
                        Frame::Integer((index + 1) as i64),  // 序号从 1 开始递增
                    ];
                    connection.write_data(Frame::Array(confirm_frames)).await?;
                }

                // streamMap 是专门为异步流设计的哈希Map，用于管理多个订阅
                let mut subscriptions = StreamMap::new();

                // 为每个频道建立订阅
                for channel in &s.channels {
                    subscribe_to_channel(db, channel, &mut subscriptions).await?;
                }

                // 监听订阅的频道和连接的消息
                loop {
                    select! {
                        // 收到某个频道的消息并处理
                        Some((channel, msg)) = subscriptions.next() => {
                            println!("Received message from channel: {}", channel);
                             let msg = vec![
                               Frame::Bulk("message".into()),
                               Frame::Bulk(channel.into()),  // 发送消息的频道
                               Frame::Bulk(msg.to_vec()),    // 订阅的消息内容
                            ];
                            // 将消息发送回客户端
                            connection.write_data(Frame::Array(msg)).await?;
                        }
                        // 接收到客户端发来的请求
                        res = connection.read_data() => {
                            let frame = match res? {
                                Some(frame) => frame,
                                // 接收订阅模式关闭信号
                                None => return Ok(()),
                            };
                            return Ok(());
                        }
                        // 处理关闭信号
                        _ = shutdown.recv() => {
                            // println!("Received shutdown signal");
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

    /// 解析客户端的 `SUBSCRIBE` 命令并返回一个 `Subscribe` 实例
    ///
    /// 该方法会解析客户端传来的频道参数并生成一个 `Subscribe` 实例，包含多个频道的列表。
    ///
    /// # 参数
    /// - `parse`: 用于解析客户端命令的 `Parse` 实例。
    ///
    /// # 返回
    /// 返回解析后的 `Subscribe` 实例，包含所有订阅的频道名称。
    /// 如果命令无效，返回错误。
    fn parse_command(parse: &mut Parse) -> crate::Result<Subscribe> {
        let mut channels = Vec::new();

        // 解析每个频道名称
        while let Ok(arg) = parse.next_string() {
            channels.push(arg);
        }

        // 如果没有传入任何频道，则返回错误
        if channels.is_empty() {
            return Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                "ERR wrong number of arguments for 'subscribe' command".to_string(),
            )));
        }

        Ok(Subscribe { channels })
    }
}

/// 为每个频道建立订阅并将接收流插入 `StreamMap` 中进行管理
///
/// 该方法会为每个频道建立一个异步消息接收流，并将其存入 `StreamMap` 中。
///
/// # 参数
/// - `db`: 用于访问数据库的共享引用。
/// - `channel`: 频道名称。
/// - `subscriptions`: 用于管理多个订阅的 `StreamMap`。
///
/// # 返回
/// 返回订阅操作的结果。
pub async fn subscribe_to_channel(
    db: &mut Arc<Mutex<Db>>,
    channel: &str,
    subscriptions: &mut StreamMap<String, crate::db::Messages>,
) -> crate::Result<()> {
    // 获取指定频道的可变引用并订阅该频道
    let mut guard = db.lock().unwrap();
    let sender = guard.subscribe(channel);

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
