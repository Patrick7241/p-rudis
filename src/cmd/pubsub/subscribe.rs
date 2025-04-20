use std::sync::{Arc, Mutex};
use crate::connection::ConnectionHandler;
use crate::db::Db;
use crate::frame::Frame;
use crate::parse::Parse;
use crate::shutdown::Shutdown;
use tokio::select;
use tokio::sync::broadcast;
use tokio_stream::{StreamExt, StreamMap};

/// `Subscribe` struct represents the subscription operation.
/// `Subscribe` 结构体用于表示订阅操作。
///
/// The struct contains a list of channels that the client wants to subscribe to.
/// 该结构体包含一个频道列表，表示客户端希望订阅的所有频道。
///
/// # Example
/// # 示例
/// ```text
/// SUBSCRIBE channel1 channel2 ...
/// ```
pub struct Subscribe {
    channels: Vec<String>,  // The list of channels to subscribe to. / 订阅的频道列表
}

impl Subscribe {
    /// Executes the `subscribe` command.
    /// 执行 `subscribe` 命令。
    ///
    /// This method will parse the incoming command and perform the subscription accordingly.
    /// 该方法将解析传入的命令并进行相应的订阅操作。
    /// It will send a confirmation message for each channel and subscribe to these channels in the background.
    /// 它将为每个频道发送确认消息，并且在后台订阅这些频道以接收消息。
    ///
    /// # Arguments
    /// # 参数
    /// - `db`: Shared reference to the database for access. / 用于访问数据库的共享引用。
    /// - `parse`: For parsing the command from the client. / 用于解析客户端传来的命令。
    /// - `connection`: The connection handler for communication with the client. / 用于与客户端进行数据通信的连接句柄。
    /// - `shutdown`: The shutdown channel for receiving shutdown signals. / 用于接收关闭信号的通道。
    ///
    /// # Return
    /// # 返回
    /// If the command is parsed successfully, returns the result of the subscription operation.
    /// 如果命令解析成功，则返回订阅操作的结果。
    pub async fn subscribe_command(
        db: &mut Arc<Mutex<Db>>,
        parse: &mut Parse,
        connection: &mut ConnectionHandler,
        shutdown: &mut Shutdown,
    ) -> crate::Result<()> {
        match Subscribe::parse_command(parse) {
            Ok(s) => {
                // Send subscription confirmation message, similar to Redis behavior
                // 发送订阅确认消息，类似于 Redis 的行为
                for (index, channel) in s.channels.iter().enumerate() {
                    let confirm_frames = vec![
                        Frame::Bulk("subscribe".into()),  // Representing the subscribe command / 表示 subscribe 命令
                        Frame::Bulk(channel.clone().into()),  // Current channel's name / 当前频道的名称
                        Frame::Integer((index + 1) as i64),  // Sequence number starts from 1 / 序号从 1 开始递增
                    ];
                    connection.write_data(Frame::Array(confirm_frames)).await?; // Send confirmation message / 发送确认消息
                }

                // `StreamMap` is designed for managing multiple subscriptions in asynchronous streams
                // streamMap 是专门为异步流设计的哈希Map，用于管理多个订阅
                let mut subscriptions = StreamMap::new();

                // Subscribe to each channel
                // 为每个频道建立订阅
                for channel in &s.channels {
                    subscribe_to_channel(db, channel, &mut subscriptions).await?; // Subscribe to the channel / 为频道订阅消息
                }

                // Listen to the subscribed channels and connection messages
                // 监听订阅的频道和连接的消息
                loop {
                    select! {
                        // Handle received message from a subscribed channel
                        // 收到某个频道的消息并处理
                        Some((channel, msg)) = subscriptions.next() => {
                            println!("Received message from channel: {}", channel);
                            let msg = vec![
                               Frame::Bulk("message".into()),  // Message type / 消息类型
                               Frame::Bulk(channel.into()),  // The channel that sent the message / 发送消息的频道
                               Frame::Bulk(msg.to_vec()),    // The content of the subscribed message / 订阅的消息内容
                            ];
                            // Send the message back to the client
                            // 将消息发送回客户端
                            connection.write_data(Frame::Array(msg)).await?; // Send message to the client / 发送消息给客户端
                        }
                        // Receive request from the client
                        // 接收到客户端发来的请求
                        res = connection.read_data() => {
                            let frame = match res? {
                                Some(frame) => frame,
                                // Receive the subscription mode close signal
                                // 接收订阅模式关闭信号
                                None => return Ok(()),
                            };
                            return Ok(());
                        }
                        // Handle shutdown signal
                        // 处理关闭信号
                        _ = shutdown.recv() => {
                            return Ok(()); // Close the connection / 关闭连接
                        }
                    }
                }
            }
            Err(_) => {
                // If parsing fails, send error message
                // 如果解析失败，发送错误信息
                connection
                    .write_data(Frame::Error(
                        "ERR wrong number of arguments for 'subscribe' command".to_string(),
                    ))
                    .await?; // Send error message / 发送错误消息
                Ok(())
            }
        }
    }

    /// Parse the client's `SUBSCRIBE` command and return a `Subscribe` instance.
    /// 解析客户端的 `SUBSCRIBE` 命令并返回一个 `Subscribe` 实例
    ///
    /// This method parses the client-provided channel parameters and generates a `Subscribe` instance
    /// with the list of channels to subscribe to.
    /// 该方法会解析客户端传来的频道参数并生成一个 `Subscribe` 实例，包含多个频道的列表。
    ///
    /// # Arguments
    /// # 参数
    /// - `parse`: The `Parse` instance used to parse the client's command. / 用于解析客户端命令的 `Parse` 实例。
    ///
    /// # Return
    /// # 返回
    /// Returns the parsed `Subscribe` instance containing all the subscribed channels.
    /// 返回解析后的 `Subscribe` 实例，包含所有订阅的频道名称。
    /// If the command is invalid, returns an error.
    /// 如果命令无效，返回错误。
    fn parse_command(parse: &mut Parse) -> crate::Result<Subscribe> {
        let mut channels = Vec::new();

        // Parse each channel name
        // 解析每个频道名称
        while let Ok(arg) = parse.next_string() {
            channels.push(arg);
        }

        // If no channels are provided, return an error
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

/// Subscribe to a channel and insert the receiving stream into `StreamMap` for management
/// 为每个频道建立订阅并将接收流插入 `StreamMap` 中进行管理
///
/// This method subscribes to a channel by creating an asynchronous message receiving stream
/// and stores it in the `StreamMap`.
/// 该方法会为每个频道建立一个异步消息接收流，并将其存入 `StreamMap` 中。
///
/// # Arguments
/// # 参数
/// - `db`: Shared reference to the database for access. / 用于访问数据库的共享引用。
/// - `channel`: The channel name. / 频道名称。
/// - `subscriptions`: The `StreamMap` that manages multiple subscriptions. / 用于管理多个订阅的 `StreamMap`。
///
/// # Return
/// # 返回
/// Returns the result of the subscription operation.
/// 返回订阅操作的结果。
pub async fn subscribe_to_channel(
    db: &mut Arc<Mutex<Db>>,
    channel: &str,
    subscriptions: &mut StreamMap<String, crate::db::Messages>,
) -> crate::Result<()> {
    // Get the mutable reference to the specified channel and subscribe to it
    // 获取指定频道的可变引用并订阅该频道
    let mut guard = db.lock().unwrap();
    let sender = guard.subscribe(channel);

    let mut receiver = sender.subscribe();

    // Create an asynchronous stream using async_stream to receive messages from the channel
    // 使用 async_stream 创建一个异步流，用于接收频道的消息
    let receiver = Box::pin(async_stream::stream! {
        loop {
            match receiver.recv().await {
                Ok(msg) => yield msg,  // Successfully received a message, forward it / 成功接收到消息，进行转发
                Err(broadcast::error::RecvError::Lagged(_)) => {},  // Skip if the message is slightly delayed / 如果接收稍有延迟，跳过
                Err(_) => break,  // Exit loop if other errors occur / 如果发生其他错误，退出循环
            }
        }
    });

    // Insert the subscribed channel and the message receiving stream into the subscriptions map
    // 将订阅的频道和对应的消息接收流插入到 subscriptions 中进行管理
    subscriptions.insert(channel.to_string(), receiver);

    // Return success
    // 返回成功
    Ok(())
}
