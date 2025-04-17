//! 解析客户端的数据


use std::io::Error;
use std::{fmt, vec};
use crate::frame::Frame;
use crate::parse::ParseError::{EndOfStream, RevertFailed, WrongType};

#[derive(Debug)]
pub struct Parse{
    /// 存储按顺序解析后的数据，利用迭代器避免额外的克隆或借用开销
    parts:vec::IntoIter<Frame>,
}
#[derive(Debug,Clone)]
pub enum ParseError{
    /// 命令结束
    EndOfStream,
    /// 转换失败
    RevertFailed,
    /// 类型错误
    WrongType
}
impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            EndOfStream => write!(f, "End of stream reached"),
            RevertFailed => write!(f, "Conversion failed"),
            WrongType => write!(f, "Wrong type encountered"),
        }
    }
}



impl std::error::Error for ParseError {}

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
    fn next(&mut self)->Result<Frame,ParseError>{
        self.parts.next().ok_or(EndOfStream)
    }
    /// 解析字节块为字符串
    pub fn next_string(&mut self) -> crate::Result<String> {
        match self.next()? {
            Frame::Simple(data) => Ok(data),
            Frame::Bulk(data) => String::from_utf8(data)
                .map_err(|_| RevertFailed.into()),
            _ => Err(WrongType.into()),
        }
    }

}
