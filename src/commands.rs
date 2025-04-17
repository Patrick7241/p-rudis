//! 命令元数据

use std::sync::Arc;
use std::sync::Mutex;
use crate::cmd;
use crate::db::Db;
use crate::frame::Frame;
use crate::parse::Parse;


/// 定义命令元数据，后续命令都可以添加到这里
pub static COMMANDS: &[(&str, &str, &str, fn(&mut Arc<Mutex<Db>>, &mut Parse) -> crate::Result<Frame>)] = &[
    // string
    ("set", "设置指定键的值。", "O(1)", cmd::string::set::Set::set_command),
    ("get", "返回指定键的字符串值。", "O(1)", cmd::string::get::Get::get_command),
];


