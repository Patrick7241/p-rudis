//! 命令元数据

use std::any::Any;
use crate::cmd;
/// 定义命令元数据，后续命令都可以添加到这里
pub static COMMANDS: &[(&str, &str, &str, fn(Option<Box<dyn Any>>) -> Option<Box<dyn Any>>)] = &[
    // string
    ("get", "返回指定键的字符串值。", "O(1)", cmd::string::get::Get::get_command),
];


