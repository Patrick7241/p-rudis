pub mod log;
pub mod cmd;

/// 定义错误返回类型
pub type Result<T>= std::result::Result<T,Box<dyn std::error::Error>>;