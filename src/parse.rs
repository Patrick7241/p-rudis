//! 解析客户端的数据
//! Parse the client's data.

use std::io::Error;
use std::{fmt, vec};
use crate::frame::Frame;
use crate::parse::ParseError::{EndOfStream, RevertFailed, WrongType};

#[derive(Debug)]
pub struct Parse{
    /// 存储按顺序解析后的数据，利用迭代器避免额外的克隆或借用开销
    /// Stores the data parsed in sequence, using an iterator to avoid extra cloning or borrowing overhead
    parts:vec::IntoIter<Frame>,
}

#[derive(Debug,Clone)]
pub enum ParseError{
    /// 命令结束
    /// Command end
    EndOfStream,
    /// 转换失败
    /// Conversion failed
    RevertFailed,
    /// 类型错误
    /// Wrong type encountered
    WrongType
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            EndOfStream => write!(f, "End of stream reached"),  // 数据流结束
            RevertFailed => write!(f, "Conversion failed"),  // 转换失败
            WrongType => write!(f, "Wrong type encountered"),  // 遇到错误类型
        }
    }
}

impl std::error::Error for ParseError {}

impl Parse{
    /// 解析客户端的命令
    /// Parse the client's command
    pub fn new(data: Option<Frame>) -> crate::Result<Parse> {
        let frame = match data {
            Some(frame) => frame,
            None => return Err(Box::new(Error::new(std::io::ErrorKind::Other, "命令为空"))),  // Command is empty
        };
        let parts = match frame {
            Frame::Array(parts) => parts,  // 如果是数组类型的命令
            _ => return Err(Box::new(Error::new(std::io::ErrorKind::Other, "命令不符合 RESP 协议规范"))),  // Command does not conform to RESP protocol
        };
        Ok(Parse{
            parts: parts.into_iter(),
        })
    }

    /// 获取下一个字节块（字节数组）
    /// Get the next byte block (byte array)
    fn next(&mut self) -> Result<Frame, ParseError> {
        self.parts.next().ok_or(EndOfStream)  // 如果没有下一个元素，返回 EndOfStream
    }

    /// 解析字节块为字符串
    /// Parse the byte block into a string
    pub fn next_string(&mut self) -> crate::Result<String> {
        match self.next()? {
            Frame::Simple(data) => Ok(data),  // 如果是简单字符串类型
            Frame::Bulk(data) => String::from_utf8(data)  // 如果是大容量字符串类型
                .map_err(|_| RevertFailed.into()),  // 如果转换失败，返回 RevertFailed 错误
            _ => Err(WrongType.into()),  // 如果是其他类型，返回 WrongType 错误
        }
    }

    /// 获取命令的参数个数（除指令外的命令数量）
    /// Get the number of arguments for the command (excluding the instruction itself)
    pub fn args_number(&mut self) -> crate::Result<usize> {
        let mut count = 0;

        let mut parts = self.parts.clone();  // 克隆 parts 以进行计数

        // 参数计数
        // Counting the arguments
        while let Some(frame) = parts.next() {
            match frame {
                Frame::Simple(_) | Frame::Bulk(_) => {
                    count += 1;  // 如果是简单字符串或大容量字符串，增加计数
                },
                _ => break,  // 如果遇到非参数类型，结束计数
            }
        }

        Ok(count)
    }
}
