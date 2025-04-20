//! 通过 RESP 协议解析命令
//! Parse commands using the RESP protocol.

use std::io::Cursor;
use bytes::{Buf, Bytes};
use atoi::atoi;
use std::fmt;

/// RESP 协议的数据类型
/// Data types for RESP protocol.
#[derive(Debug,Clone)]
pub enum Frame{
    /// 简单字符串，如 +OK 或 +PONG 等简单回复
    /// Simple strings, such as +OK or +PONG, etc.
    Simple(String),
    /// 错误类型，如 -ERR unknown command
    /// Error type, such as -ERR unknown command
    Error(String),
    /// 整数类型，如 :1 或 :1000 等
    /// Integer type, such as :1 or :1000, etc.
    Integer(i64),
    /// 大容量字符串，如 $5\r\nhello\r\n
    /// Bulk strings, such as $5\r\nhello\r\n
    Bulk(Vec<u8>),
    /// 空值或 null，通常在某些命令的返回值为空时出现
    /// Null value, typically when some command's return value is empty.
    Null,
    /// 数组类型，如 *2\r\n$3\r\nfoo\r\n$3\r\nbar\r\n 表示一个包含两个元素的数组
    /// Array type, such as *2\r\n$3\r\nfoo\r\n$3\r\nbar\r\n, representing an array with two elements.
    Array(Vec<Frame>),

    /// 非 RESP协议类型
    /// Non-RESP protocol type
    /// 用于表示不需要统一函数返回的回复，由调用者自行处理回复。
    /// This is used to indicate a reply that does not need a unified return function and is handled by the caller.
    NoResponse,
}

#[derive(Debug)]
pub enum Error{
    /// 没有更多的数据可以读
    /// No more data to read
    NoMoreData,
    /// 不是数字
    /// Not a number
    NotNumber,
    /// 溢出
    /// Overflow
    OverFlow,
    /// 类型转化错误
    /// Type conversion error
    TypeConversionError,
    /// 不符合 RESP 协议
    /// Does not conform to RESP protocol
    UnRESP
}

/// 实现 std::error::Error 的trait，以使用？运算符
/// Implement the std::error::Error trait to use the ? operator
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::NoMoreData => write!(f, "没有更多的数据可以读取"),  // No more data to read
            Error::NotNumber => write!(f, "值不是数字"),  // Value is not a number
            Error::OverFlow => write!(f, "发生了溢出错误"),  // Overflow error occurred
            Error::TypeConversionError => write!(f, "类型转换错误"),  // Type conversion error
            Error::UnRESP => write!(f, "数据不符合 RESP 协议"),  // Data does not conform to RESP protocol
        }
    }
}

impl std::error::Error for Error {}

impl Frame {
    /// 检查命令是否符合 resp 协议规范，不实际处理命令，也不是会实际检查指令是否正确，比如set test也能过，只会检查是否遵循resp协议
    /// Check if the command adheres to the RESP protocol specification. It does not process the command itself, nor does it check if the instruction is correct (e.g., "set test" is allowed); it only checks if the RESP protocol is followed.
    /// 使用Cursor更方便和高效的操作字节流
    /// Use `Cursor` for more convenient and efficient byte stream handling.
    pub fn check(command: &mut Cursor<&[u8]>) -> Result<(), Error> {
        match get_bytes(command)? {
            b'*' => {
                // 获取*后面的数字，并更新当前字节位置（在get_line函数里面的操作）
                // Get the number after '*' and update the byte position (done in the get_line function).
                let number = get_number(command)?;
                // 循环检查每一行是否符合要求
                // Loop through and check each line.
                for _ in 0..number {
                    Frame::check(command)?;
                }
                Ok(())
            }
            b'$' => {
                if b'-' == peek_bytes(command)? {
                    // 跳过-1\r\n
                    // Skip "-1\r\n".
                    skip_bytes(command, 4)
                } else {
                    let length: usize = get_number(command)?
                        .try_into()
                        .map_err(|_| Error::TypeConversionError)?;
                    // 跳过对应长度，+2是跳过\r\n
                    // Skip the corresponding length, +2 to skip "\r\n".
                    skip_bytes(command, length + 2)
                }
            }
            b':' => {
                // 检查:后面有没用数字
                // Check if there is a number after ':'.
                get_number(command)?;
                Ok(())
            }
            b'+' => {
                // 检查+后面有没有简单字符串
                // Check if there is a simple string after '+'.
                get_line(command)?;
                Ok(())
            }
            b'-' => {
                // 检查-后面有没有简单字符串
                // Check if there is a simple string after '-'.
                get_line(command)?;
                Ok(())
            }
            _ => {
                // TODO 读取字节流的错误处理或者读取完毕处理
                // TODO: Error handling for reading byte streams or handling completion.
                Ok(())
            }
        }
    }

    /// 解析命令，并返回解析结果
    /// Parse the command and return the parsed result.
    pub fn parse(command: &mut Cursor<&[u8]>) -> Result<Frame, Error> {
        match get_bytes(command)? {
            b'*' => {
                let number = get_number(command)? as usize;
                let mut frames = Vec::with_capacity(number);
                for _ in 0..number {
                    frames.push(Frame::parse(command)?);
                }
                Ok(Frame::Array(frames))
            }
            b'$' => {
                if b'-' == peek_bytes(command)? {
                    let line = get_line(command)?;
                    // 如果不是-1，就不是resp协议规定的返回类型，返回错误
                    // If it's not "-1", it's not a valid RESP protocol return type, return error.
                    if line != b"-1" {
                        return Err(Error::UnRESP);
                    }
                    Ok(Frame::Null)
                } else {
                    // 读取长度信息
                    // Read the length information.
                    let len = get_number(command)? as usize;
                    let n = len + 2;
                    if n > command.remaining() {
                        return Err(Error::NoMoreData);
                    }
                    let data = Bytes::copy_from_slice(&command.chunk()[..len]);
                    skip_bytes(command, n)?;
                    Ok(Frame::Bulk(data.to_vec()))
                }
            }
            b':' => {
                // 返回整数
                // Return integer.
                let number = get_number(command)?;
                Ok(Frame::Integer(number))
            }
            b'+' => {
                // 返回简单字符串
                // Return simple string.
                let line = String::from_utf8(get_line(command)?.to_vec())
                    .map_err(|_| Error::TypeConversionError)?;

                Ok(Frame::Simple(line))
            }
            b'-' => {
                // 返回简单字符串
                // Return simple string.
                let line = String::from_utf8(get_line(command)?.to_vec())
                    .map_err(|_| Error::TypeConversionError)?;

                Ok(Frame::Simple(line))
            }
            _ => {
                // TODO 读取字节流的错误处理或者读取完毕处理
                // TODO: Error handling for reading byte streams or handling completion.
                Ok(Frame::Null)
            }
        }
    }

    /// 将frame转化为resp格式的bytes，返回客户端
    /// Convert the frame to RESP format bytes to return to the client.
    pub fn to_bytes(&self) -> Option<Vec<u8>> {
        match self {
            // 处理 Simple 类型
            // Handle Simple type
            Frame::Simple(s) => {
                let mut bytes = Vec::new();
                bytes.push(b'+'); // +符号
                bytes.extend_from_slice(s.as_bytes()); // 添加字符串内容
                bytes.extend_from_slice(b"\r\n"); // 添加换行符
                Some(bytes)
            },

            // 处理 Error 类型
            // Handle Error type
            Frame::Error(e) => {
                let mut bytes = Vec::new();
                bytes.push(b'-'); // -符号
                bytes.extend_from_slice(e.as_bytes()); // 添加错误信息
                bytes.extend_from_slice(b"\r\n"); // 添加换行符
                Some(bytes)
            },

            // 处理 Integer 类型
            // Handle Integer type
            Frame::Integer(i) => {
                let mut bytes = Vec::new();
                bytes.push(b':'); // :符号
                bytes.extend_from_slice(i.to_string().as_bytes()); // 转换整数为字符串并添加
                bytes.extend_from_slice(b"\r\n"); // 添加换行符
                Some(bytes)
            },

            // 处理 Bulk 类型
            // Handle Bulk type
            // 格式: $<长度>\r\n<数据>\r\n
            // Format: $<length>\r\n<data>\r\n
            Frame::Bulk(data) => {
                let mut frame = Vec::new();
                frame.extend_from_slice(format!("${}\r\n", data.len()).as_bytes());
                frame.extend_from_slice(data);
                frame.extend_from_slice(b"\r\n");
                Some(frame)
            }

            // 处理 Null 类型
            // Handle Null type
            Frame::Null => {
                let mut bytes = Vec::new();
                bytes.push(b'$'); // $符号
                bytes.push(b'-'); // -符号，表示空值
                bytes.extend_from_slice(b"1"); // 长度为 1
                bytes.extend_from_slice(b"\r\n"); // 换行符
                Some(bytes)
            },

            // 处理 Array 类型
            // Handle Array type
            Frame::Array(arr) => {
                let mut bytes = Vec::new();
                bytes.push(b'*'); // *符号，表示数组类型
                bytes.extend_from_slice(arr.len().to_string().as_bytes()); // 数组长度
                bytes.extend_from_slice(b"\r\n"); // 换行符
                for frame in arr {
                    if let Some(mut frame_bytes) = frame.to_bytes() {
                        bytes.append(&mut frame_bytes); // 将每个元素的字节追加到数组
                    }
                }
                Some(bytes)
            },

            // 捕获其他未处理类型
            // Capture other unhandled types
            _ => None,
        }
    }
}

/// 跳过指定数量的字节
/// Skip the specified number of bytes.
fn skip_bytes(command: &mut Cursor<&[u8]>, n: usize) -> Result<(), Error> {
    if !command.has_remaining() {
        return Err(Error::NoMoreData);
    }
    command.advance(n);
    Ok(())
}

/// 获取第一个字节，但不移动cursor位置
/// Get the first byte without moving the cursor position.
fn peek_bytes(command: &mut Cursor<&[u8]>) -> Result<u8, Error> {
    if !command.has_remaining() {
        return Err(Error::NoMoreData);
    }
    Ok(command.chunk()[0])
}

/// 获取*后面的数据，并判断是否为数字
/// Get the number after '*' and check if it's a number.
fn get_number(command: &mut Cursor<&[u8]>) -> Result<i64, Error> {
    let line = get_line(command)?;
    match atoi::<i64>(line) {
        Some(num) => {
            Ok(num)
        }
        None => {
            Err(Error::NotNumber)
        }
    }
}

/// 获取一行的数据，根据 \r\n 分割，并更新cursor游标位置
/// Get a line of data, split by \r\n, and update the cursor position.
fn get_line<'a>(command: &mut Cursor<&'a [u8]>) -> Result<&'a [u8], Error> {
    let start = command.position() as usize;
    let end = command.get_ref().len() - 1;
    for i in start..end {
        if command.get_ref()[i] == b'\r' && command.get_ref()[i + 1] == b'\n' {
            command.set_position((i + 2) as u64);
            let bytes = &command.get_ref()[start..i];
            return Ok(bytes);
        }
    }
    Err(Error::NoMoreData)
}

/// 获取字节流中的一个字节并移动游标位置
/// Get a byte from the byte stream and move the cursor position.
fn get_bytes(command: &mut Cursor<&[u8]>) -> Result<u8, Error> {
    if !command.has_remaining() {
        return Err(Error::NoMoreData);
    }
    Ok(command.get_u8())
}
