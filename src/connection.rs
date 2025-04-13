//! 处理与客户端的连接

use std::io::{Cursor, Error};
use tokio::net::TcpStream;
use bytes::{Buf, BytesMut};
use log::{error, info};
use tokio::io::AsyncReadExt;
use crate::frame::Frame;

#[derive(Debug)]
pub struct ConnectionHandler {
    /// TCP连接对象
    stream: TcpStream,
    /// 缓冲区
    buffer: BytesMut,
}

impl ConnectionHandler {
    /// 定义一个连接，设置 1024 字节大小缓冲区，根据需要可适当扩容
    pub fn new(stream: TcpStream) -> Self {
        ConnectionHandler {
            stream,
            buffer: BytesMut::with_capacity(1024),
        }
    }

    /// 读取客户端发送的数据
    pub async fn read_data(&mut self) -> p_rudis::Result<Option<Frame>> {
        loop {
            // 从流中读取数据到缓冲区
            match self.stream.read_buf(&mut self.buffer).await {
                Ok(0) => {
                   info!("客户端断开连接");
                    // 清理缓冲区
                    self.buffer.clear();
                    return Err(Box::new(Error::new(std::io::ErrorKind::Other, "客户端断开连接")))
                }
                Ok(n) => {
                    if n > 0 {
                        // 解析读取到的数据
                       if let Some(data)= self.parse_data(n)?{
                           return Ok(Some(data))
                       }
                    }
                }
                Err(err) => {
                    // 读取数据出错
                    info!("读取数据出错: {}", err);
                    // 清理缓冲区
                    self.buffer.clear();
                    return Err(Box::new(err));
                }
            }
        }
    }
    /// 解析数据
    fn parse_data(&mut self,n:usize)->p_rudis::Result<Option<Frame>>{
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


}
