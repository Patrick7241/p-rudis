//! 解析客户端的数据

use std::fmt::format;
use std::io::Error;
use std::vec;
use crate::frame::Frame;

#[derive(Debug)]
pub struct Parse{
    /// 存储按顺序解析后的数据，利用迭代器避免额外的克隆或借用开销
    parts:vec::IntoIter<Frame>,
}

impl Parse{
    /// 解析客户端的命令
    pub fn new(data: Option<Frame>)-> crate::Result<Parse> {
        let frame=match data {
            Some(frame)=>frame,
            None=>return Err(Box::new(Error::new(std::io::ErrorKind::Other, "命令为空"))),
        };
        let parts=match frame {
            Frame::Array(parts)=>parts,
            _=>return Err(Box::new(Error::new(std::io::ErrorKind::Other, "命令不符合 RESP 协议规范"))),
        };
        Ok(Parse{
            parts:parts.into_iter(),
        })
    }
    /// 获取下一个字节块（字节数组）
    fn next(&mut self)->Result<Frame,Error>{
        self.parts.next().ok_or(Error::new(std::io::ErrorKind::Other, "命令结束"))
    }
    /// 解析字节块为字符串
    pub fn next_string(&mut self)->crate::Result<String>{
        match self.next()? {
            // 简单类型，直接返回字符串
            Frame::Simple(data)=>
                Ok(data),
            // 字节数组，转成字符串
            Frame::Bulk(data)=>
                Ok(String::from_utf8(data)?),
            // 不符合类型，返回错误
            frame => Err(Box::new(Error::new(
                std::io::ErrorKind::InvalidData,
                format!("期望Simple类型或Bulk类型，实际获得{:?}类型", frame),
            ))),
        }
    }

}
