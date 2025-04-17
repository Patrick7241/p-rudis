//! 通过 RESP 协议解析命令

use std::io::Cursor;
use bytes::{Buf, Bytes};
use atoi::atoi;
use std::fmt;

/// RESP 协议的数据类型
#[derive(Debug)]
pub enum Frame{
    /// 简单字符串，如 +OK 或 +PONG 等简单回复
    Simple(String),
    /// 错误类型，如 -ERR unknown command
    Error(String),
    /// 整数类型，如 :1 或 :1000 等
    Integer(i64),
    /// 大容量字符串，如 $5\r\nhello\r\n
    Bulk(Vec<u8>),
    /// 空值或 null，通常在某些命令的返回值为空时出现
    Null,
    /// 数组类型，如 *2\r\n$3\r\nfoo\r\n$3\r\nbar\r\n 表示一个包含两个元素的数组
    Array(Vec<Frame>),
}
#[derive(Debug)]
pub enum Error{
    /// 没用更多的数据可以读
    NoMoreData,
    /// 不是数字
    NotNumber,
    /// 溢出
    OverFlow,
    /// 类型转化错误
    TypeConversionError,
    /// 不符合 RESP 协议
    UnRESP
}
/// 实现 std::error::Error 的trait，以使用？运算符
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::NoMoreData => write!(f, "没有更多的数据可以读取"),
            Error::NotNumber => write!(f, "值不是数字"),
            Error::OverFlow => write!(f, "发生了溢出错误"),
            Error::TypeConversionError => write!(f, "类型转换错误"),
            Error::UnRESP => write!(f, "数据不符合 RESP 协议"),
        }
    }
}

impl std::error::Error for Error {}


impl Frame{
    /// 检查命令是否符合 resp 协议规范，不实际处理命令，也不是会实际检查指令是否正确，比如set test也能过，只会检查是否遵循resp协议
    /// 使用Cursor更方便和高效的操作字节流
    pub fn check(command:&mut Cursor<&[u8]>)->Result<(),Error>{
        match get_bytes(command)? {
            b'*'=>{
                // 获取*后面的数字，并更新当前字节位置（在get_line函数里面的操作）
                let number=get_number(command)?;
                // 循环检查每一行是否符合要求
                for _ in 0..number{
                    Frame::check(command)?;
                }
                Ok(())
            }
            b'$'=>{
                if b'-'==peek_bytes(command)?{
                    // 跳过-1\r\n
                    skip_bytes(command,4)
                }else{
                    let length:usize=get_number(command)?
                        .try_into()
                        .map_err(|_|Error::TypeConversionError)?;
                    // 跳过对应长度，+2是跳过\r\n
                    skip_bytes(command,length+2)
                }
            }
            b':'=>{
                // 检查:后面有没用数字
                get_number(command)?;
                Ok(())
            }
            b'+'=>{
                // 检查+后面有没有简单字符串
                get_line(command)?;
                Ok(())
            }
            b'-'=>{
                // 检查-后面有没有简单字符串
                get_line(command)?;
                Ok(())
            }
           _=>{
                // TODO 读取字节流的错误处理或者读取完毕处理
               Ok(())
            }
        }
    }

    /// 解析命令，并返回解析结果
    pub fn parse(command:&mut Cursor<&[u8]>)->Result<Frame,Error>{
        match get_bytes(command)? {
            b'*'=>{
                let number=get_number(command)? as usize;
                let mut frames=Vec::with_capacity(number);
                for _ in 0..number{
                    frames.push(Frame::parse(command)?);
                }
                Ok(Frame::Array(frames))
            }
            b'$'=>{
                if b'-'==peek_bytes(command)?{
                    let line=get_line(command)?;
                    // 如果不是-1，就不是resp协议规定的返回类型，返回错误
                    if line!=b"-1"{
                        return Err(Error::UnRESP)
                    }
                   Ok(Frame::Null)
                }else{
                    // 读取长度信息
                    let len = get_number(command)? as usize;
                    let n=len+2;
                    if n>command.remaining(){
                        return Err(Error::NoMoreData);
                    }
                    let data=Bytes::copy_from_slice(&command.chunk()[..len]);
                    skip_bytes(command,n)?;
                    Ok(Frame::Bulk(data.to_vec()))
                }
            }
            b':'=>{
                // 返回整数
                let number=get_number(command)?;
                Ok(Frame::Integer(number))
            }
            b'+'=>{
                // 返回简单字符串
                let line = String::from_utf8(get_line(command)?.to_vec())
                    .map_err(|_| Error::TypeConversionError)?;

                Ok(Frame::Simple(line))
            }
            b'-'=>{
                // 返回简单字符串
                let line = String::from_utf8(get_line(command)?.to_vec())
                    .map_err(|_| Error::TypeConversionError)?;

                Ok(Frame::Simple(line))
            }
            _=>{
                // TODO 读取字节流的错误处理或者读取完毕处理
                Ok(Frame::Null)
            }
        }
    }

    /// 将frame转化为resp格式的bytes，返回客户端
    pub fn to_bytes(&self) -> Option<Vec<u8>> {
        match self {
            // 处理 Simple 类型
            Frame::Simple(s) => {
                let mut bytes = Vec::new();
                bytes.push(b'+'); // +符号
                bytes.extend_from_slice(s.as_bytes()); // 添加字符串内容
                bytes.extend_from_slice(b"\r\n"); // 添加换行符
                Some(bytes)
            },

            // 处理 Error 类型
            Frame::Error(e) => {
                let mut bytes = Vec::new();
                bytes.push(b'-'); // -符号
                bytes.extend_from_slice(e.as_bytes()); // 添加错误信息
                bytes.extend_from_slice(b"\r\n"); // 添加换行符
                Some(bytes)
            },

            // 处理 Integer 类型
            Frame::Integer(i) => {
                let mut bytes = Vec::new();
                bytes.push(b':'); // :符号
                bytes.extend_from_slice(i.to_string().as_bytes()); // 转换整数为字符串并添加
                bytes.extend_from_slice(b"\r\n"); // 添加换行符
                Some(bytes)
            },

            // 处理 Bulk 类型
            // 格式: $<长度>\r\n<数据>\r\n
            Frame::Bulk(data) => {
                let mut frame = Vec::new();
                frame.extend_from_slice(format!("${}\r\n", data.len()).as_bytes());
                frame.extend_from_slice(data);
                frame.extend_from_slice(b"\r\n");
                Some(frame)
            }

            // 处理 Null 类型
            Frame::Null => {
                let mut bytes = Vec::new();
                bytes.push(b'$'); // $符号
                bytes.push(b'-'); // -符号，表示空值
                bytes.extend_from_slice(b"1"); // 长度为 1
                bytes.extend_from_slice(b"\r\n"); // 换行符
                Some(bytes)
            },

            // 处理 Array 类型
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
            _ => None,
        }
    }


}

/// 跳过指定数量的字节
fn skip_bytes(command :&mut Cursor<&[u8]>,n:usize)->Result<(),Error>{
    if !command.has_remaining(){
        return Err(Error::NoMoreData);
    }
    // let position=command.position() as usize;
    // if position+n>command.get_ref().len(){
    //     return Err(Error::OverFlow);
    // }
    // command.set_position((position+n) as u64);

    // advance的内部实现是set_position，但是会有越界和加法溢出检查
    command.advance(n);
    Ok(())
}

/// 获取第一个字节，但不移动cursor位置
fn peek_bytes(command :&mut Cursor<&[u8]>)->Result<u8,Error>{
    if !command.has_remaining(){
        return Err(Error::NoMoreData);
    }
    Ok(command.chunk()[0])
}

/// 获取*后面的数据，并判断是否为数字
fn get_number(command: &mut Cursor<&[u8]>)->Result<i64,Error>{
    let line = get_line(command)?;
    // 尝试将字节切片解析为数字
    match atoi::<i64>(line) {
        Some(num) => {
            Ok(num)
        }
        None => {
            // 如果解析失败，返回错误
            Err(Error::NotNumber)
        }
    }
}

/// 获取一行的数据，根据 \r\n 分割，并更新cursor游标位置
/// 'a 是一个泛型生命周期，表示一个引用的生命周期由函数参数的生命周期决定，通常在函数签名中使用来确保返回的引用与输入的引用生命周期一致
fn get_line<'a>(command:&mut Cursor<&'a [u8]>)->Result<&'a [u8],Error>{
    // 获取当前位置
    let start=command.position() as usize;
    // 获取结束位置
    let end=command.get_ref().len()-1;
    // 遍历字节
    for i in start..end{
        // 以 \r\n 为结束
        if command.get_ref()[i]==b'\r'&& command.get_ref()[i+1]==b'\n'{
            // 更新当前位置
            command.set_position((i+2) as u64);
            // 获取字节
            let bytes=&command.get_ref()[start..i];
            return Ok(bytes)
        }
    }
    Err(Error::NoMoreData)
}

fn get_bytes(command:&mut Cursor<&[u8]>)->Result<u8,Error>{
    if !command.has_remaining(){
        return Err(Error::NoMoreData)
    }
    // get_u8的内部是advance，也是set_position
    Ok(command.get_u8())
}