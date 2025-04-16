//! 定义统一处理类型或其他


/// 定义错误返回类型
pub type Result<T>= std::result::Result<T,Box<dyn std::error::Error+Send+Sync>>;