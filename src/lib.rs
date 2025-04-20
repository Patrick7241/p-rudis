//! 定义统一处理类型

pub mod server;
pub mod connection;
pub mod commands;
pub mod db;
pub mod frame;
pub mod shutdown;
pub mod log;
pub mod dict;
pub mod parse;
pub mod cmd;


/// 定义错误返回类型
/// This defines a custom result type that can be used throughout the application.
/// 它定义了一个自定义的返回类型，可以在整个应用程序中使用。
/// This allows for more flexible error handling by returning a result wrapped in a `Box` to handle various types of errors.
/// 通过将结果包装在一个 `Box` 中来处理各种类型的错误，从而实现更灵活的错误处理。
pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

