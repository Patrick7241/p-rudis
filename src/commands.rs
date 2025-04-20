//! 命令元数据
/// Command metadata

use std::sync::Arc;
use std::sync::Mutex;
use crate::cmd;
use crate::db::Db;
use crate::frame::Frame;
use crate::parse::Parse;

/// 空函数占位，不符合统一参数和返回值标准的函数可使用
/// Placeholder function for commands that don't match the standard argument and return value conventions.
fn empty_command(_: &mut Arc<Mutex<Db>>, _: &mut Parse) -> crate::Result<Frame> {
    Ok(Frame::NoResponse)
}

/// 定义命令元数据，后续命令都可以添加到这里
/// Define command metadata, additional commands can be added here in the future.
pub static COMMANDS: &[(&str, &str, &str, fn(&mut Arc<Mutex<Db>>, &mut Parse) -> crate::Result<Frame>)] = &[
    // ping
    ("ping", "测试连接是否正常。", "O(1)", cmd::ping::Ping::ping_command),
    // echo
    ("echo", "返回指定的字符串。", "O(N)", cmd::echo::Echo::echo_command),
    // pubsub
    ("publish", "向指定频道发布消息。", "O(1)", cmd::pubsub::publish::Publish::publish_command),
    ("subscribe", "订阅指定频道，接收消息。", "O(1)", empty_command),
    ("psubscribe", "使用模式订阅频道。", "O(1)", empty_command),
    // string
    ("set", "设置指定键的值。", "O(1)", cmd::string::set::Set::set_command),
    ("get", "返回指定键的字符串值。", "O(1)", cmd::string::get::Get::get_command),
    ("del", "删除指定的键。", "O(1)", cmd::string::del::Del::del_command),
    ("append", "将指定的值追加到键的字符串值后面。", "O(1)", cmd::string::append::Append::append_command),
    ("strlen", "获取指定键的字符串值的长度。", "O(1)", cmd::string::strlen::Strlen::strlen_command),
    ("incr", "将指定键的数值增加1。", "O(1)", cmd::string::incr::Incr::incr_command),
    ("incrby", "将指定键的数值增加指定的步长，无默认值。", "O(1)", cmd::string::incrby::IncrBy::incrby_command),
    ("decr", "将指定键的数值减少1。", "O(1)", cmd::string::decr::Decr::decr_command),
    ("decrby", "将指定键的数值减少指定的步长，无默认值。", "O(1)", cmd::string::decrby::DecrBy::decrby_command),
    ("mget", "获取多个指定键的字符串值。", "O(N)", cmd::string::mget::Mget::mget_command),
    ("mset", "设置多个键的值。", "O(N)", cmd::string::mset::Mset::mset_command),
    ("msetnx", "只有在所有指定键都不存在的情况下，才会设置它们的值。", "O(N)", cmd::string::msetnx::Msetnx::msetnx_command),
    // hash
    ("hset", "设置哈希表中指定字段的值。", "O(1)", cmd::hash::hset::Hset::hset_command),
    ("hget", "获取哈希表中指定字段的值。", "O(1)", cmd::hash::hget::Hget::hget_command),
    ("hdel", "删除哈希表中指定字段。", "O(1)", cmd::hash::hdel::Hdel::hdel_command),
    ("hgetall", "获取哈希表中的所有字段和值。", "O(N)", cmd::hash::hgetall::Hgetall::hgetall_command),
    ("hmset", "设置哈希表中多个字段的值。", "O(N)", cmd::hash::hmset::Hmset::hmset_command),
    ("hmget", "获取哈希表中多个字段的值。", "O(N)", cmd::hash::hmget::Hmget::hmget_command),
    ("hkeys", "获取哈希表中的所有字段。", "O(N)", cmd::hash::hkeys::Hkeys::hkeys_command),
    ("hvals", "获取哈希表中的所有值。", "O(N)", cmd::hash::hvals::Hvals::hvals_command),
    ("hlen", "获取哈希表中的字段数量。", "O(1)", cmd::hash::hlen::Hlen::hlen_command),
    ("hexists", "检查哈希表中指定字段是否存在。", "O(1)", cmd::hash::hexists::Hexists::hexists_command),
    ("hsetnx", "只有在字段不存在的情况下，才会设置字段的值。", "O(1)", cmd::hash::hsetnx::Hsetnx::hsetnx_command),
    // list
    ("lpush", "将一个或多个值插入到列表的头部。", "O(1)", cmd::list::lpush::Lpush::lpush_command),
    ("rpush", "将一个或多个值插入到列表的尾部。", "O(1)", cmd::list::rpush::Rpush::rpush_command),
    ("lpop", "移除并返回列表的第一个元素。", "O(1)", cmd::list::lpop::Lpop::lpop_command),
    ("rpop", "移除并返回列表的最后一个元素。", "O(1)", cmd::list::rpop::Rpop::rpop_command),
    ("lrange", "返回列表中指定范围的元素。", "O(N)", cmd::list::lrange::Lrange::lrange_command),
    ("lindex", "返回列表中指定索引的元素。", "O(1)", cmd::list::lindex::Lindex::lindex_command),
    // ("llen", "返回列表的长度。", "O(1)", cmd::list::llen::Llen::llen_command),
    // ("lset", "设置列表中指定索引的值。", "O(N)", cmd::list::lset::Lset::lset_command),
    // ("lrem", "移除列表中指定值的元素。", "O(N)", cmd::list::lrem::Lrem::lrem_command),
    // ("ltrim", "对列表进行修剪，保留指定范围的元素。", "O(N)", cmd::list::ltrim::Ltrim::ltrim_command),
    // ("blpop", "阻塞式从左侧弹出一个元素。", "O(1)", cmd::list::blpop::Blpop::blpop_command),
    // ("brpop", "阻塞式从右侧弹出一个元素。", "O(1)", cmd::list::brpop::Brpop::brpop_command),
    // ("brpoplpush", "阻塞式弹出一个元素并将其推入另一个列表。", "O(1)", cmd::list::brpoplpush::Brpoplpush::brpoplpush_command)
];
