//! 命令元数据

use crate::cmd;
use crate::db::Db;
use crate::parse::Parse;


/// 定义命令元数据，后续命令都可以添加到这里
pub static COMMANDS: &[(&str, &str, &str, fn(&mut Db, &mut Parse) -> crate::Result<String>)] = &[
    // string
    ("get", "返回指定键的字符串值。", "O(1)", cmd::string::get::Get::get_command),
];


