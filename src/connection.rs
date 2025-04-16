//! 处理与客户端的连接，接收和返回消息

use std::io::{Cursor, Error};
use std::sync::{Arc};
use tokio::net::TcpStream;
use bytes::{Buf, BytesMut};
use log::{error, info};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::sync::Mutex;
use crate::frame::Frame;

#[derive(Debug,Clone)]
pub struct ConnectionHandler {
    /// TCP连接对象
    stream: Arc<Mutex<TcpStream>>,
    /// 缓冲区
    buffer: BytesMut,
}

impl ConnectionHandler {
    /// 定义一个连接，设置 1024 字节大小缓冲区，根据需要可适当扩容
    pub fn new(stream:  Arc<Mutex<TcpStream>>) -> Self {
        ConnectionHandler {
            stream,
            buffer: BytesMut::with_capacity(1024),
        }
    }

    /// 读取客户端发送的数据
    pub async fn read_data(&mut self) -> crate::Result<Option<Frame>> {
        loop {
            // 限制 `MutexGuard` 的作用域，避免它在调用 `parse_data` 时仍然存活
            // 在单独的块中处理锁的获取和释放，解决 MutexGuard 锁住变量导致后面不能可变借用的问题
            let n = {
                let mut stream = self.stream.lock().await;
                // 从流中读取数据到缓冲区
                match stream.read_buf(&mut self.buffer).await {
                    Ok(0) => {
                        // 清理缓冲区
                        self.buffer.clear();
                        return Err(Box::new(Error::new(std::io::ErrorKind::Other, "客户端断开连接")));
                    }
                    Ok(n) => n,
                    Err(err) => {
                        // 清理缓冲区
                        self.buffer.clear();
                        return Err(Box::new(err));
                    }
                }
            };

            if n > 0 {
                if let Some(data) = self.parse_data(n)? {
                    return Ok(Some(data));
                }
            }
        }
    }
    /// 解析数据
    fn parse_data(&mut self,n:usize)->crate::Result<Option<Frame>>{
        let mut command=Cursor::new(&self.buffer[..n]);

        // 检查命令是否符合 resp 协议规范
        match Frame::check(&mut command) {
            Ok(_)=>{
                // 获取当前游标位置，因为check后游标会被
                // 移动到最末端，所以当前位置也是数据大小
                let len=command.position() as usize;
                // 重置游标位置
                command.set_position(0);
                // 命令符合 RESP 协议规范，开始解析数据
                let frame=Frame::parse(&mut command)?;
                // 移动游标位置，删除已经解析的数据
                self.buffer.advance(len);
                Ok(Some(frame))
            }
            Err(err)=>{
                error!("命令不符合 RESP 协议规范: {:?}", err);
                Err(Box::new(Error::new(std::io::ErrorKind::Other, "命令不符合 RESP 协议规范")))
            }
        }
    }

    /// 发送回复消息到客户端
    /// TODO 测试方便，先不解析resp，只传输字符串
    pub async fn write_data(&mut self, response: String) -> crate::Result<()> {
        info!("测试内容！！！：非resp协议，发送回复消息到客户端: {}", response);
        // 将字符串转换为字节数组
        let bytes = response.as_bytes();
        // 将字节数组写入流中
        self.stream.lock().await.write_all(bytes).await?;
        // 刷新流，确保数据立即发送
        self.stream.lock().await.flush().await?;
        Ok(())
    }
}
