//! 命令元数据

use std::any::Any;
use crate::cmd;
use crate::connection::ConnectionHandler;
use crate::db::Db;
use crate::parse::Parse;

/// 定义命令元数据，后续命令都可以添加到这里
pub static COMMANDS: &[(&str, &str, &str, fn(Db, ConnectionHandler, Parse) -> crate::Result<()>)] = &[
    // string
    ("get", "返回指定键的字符串值。", "O(1)", cmd::string::get::Get::get_command),
];


