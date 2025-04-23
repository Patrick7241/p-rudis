use std::collections::{HashMap, HashSet, VecDeque};
use std::fs::File;
use std::io;
use std::io::Write;
use std::pin::Pin;
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use std::sync::Mutex;
use tokio::sync::broadcast;
use tokio_stream::{Stream, StreamExt};
use bytes::Bytes;
use crate::persistence::aof::propagate_aof;

/// 定义一个类型别名 Messages，表示一个动态的异步流。
/// 这个异步流用于处理字节数据（Bytes），并且可以跨线程安全地传递。
/// 使用 Pin<Box<dyn Stream<Item = Bytes> + Send>> 的原因如下：
/// - dyn Stream<Item = Bytes>：允许动态地处理不同类型的异步流，
///   只要它们产生的数据类型是 Bytes。这提供了灵活性，可以支持多种数据源。
/// - Pin<Box<...>>：确保异步流在内存中的位置不会改变。这是异步运行时（如 Tokio）
///   的要求，以避免悬挂指针或其他内存安全问题。
/// - + Send：确保这个异步流可以在多个线程之间安全地传递，这是并发编程中的一个重要特性。
pub(crate) type Messages = Pin<Box<dyn Stream<Item = Bytes> + Send>>;

#[derive(Debug)]
pub struct DbHolder {
    db: Arc<Mutex<Db>>,
}
#[derive(Clone, Debug)]
pub struct Db {
    storage: HashMap<String, DbEntry>,
    /// 发布/订阅模式
    /// A publish/subscribe model, where the key is the channel and the value is the broadcast sender for that channel.
    pub_sub: HashMap<String, broadcast::Sender<Bytes>>,
    /// 记录发布/订阅模式下，通配符的广播
    /// Records the broadcast for the publish/subscribe pattern with wildcard.
    psubscribes: HashMap<String, broadcast::Sender<Bytes>>,
}

#[derive(Clone, Debug)]
pub struct DbEntry {
    /// 基本数据结构的数据类型
    /// The data type of the basic structure.
    pub(crate)  value: DbType,
    /// 存储过期时间，单位 毫秒
    /// The expiration time of the entry, in milliseconds.
    pub(crate) expiration: Option<u64>,
}

#[derive(Clone, Debug)]
pub enum DbType {
    String(String),
    Hash(HashMap<String, String>),
    List(VecDeque<String>),
    // Set(HashSet<String>),
    // ZSet(String),  // 有序集合
    // BitMap(String), // 位图
}

impl DbHolder {
    pub fn new() -> Self {
        Self {
            db: Arc::new(Mutex::new(Db::new())),
        }
    }

    pub fn get_db(&self) -> Arc<Mutex<Db>> {
        self.db.clone()
    }
}

impl Db {
    pub fn new() -> Db {
        let db = Db {
            storage: HashMap::new(),
            pub_sub: HashMap::new(),
            psubscribes: HashMap::new(),
        };
        // 开启定时任务，定时处理过期的键值
        // Start a periodic task to clean up expired keys.
        tokio::spawn(periodic_cleanup(db.clone(), Duration::from_secs(60)));
        db
    }
    pub fn iter(&self) -> impl Iterator<Item = (&String, &DbEntry)> {
        self.storage.iter()
    }

    /// 获取DbType的可变引用
    /// Get a mutable reference to the DbType of a given key.
    pub fn get_dbtype_mut(&mut self, key: &str) -> Option<&mut DbType> {
        match self.storage.get_mut(key) {
            Some(entry) => Some(&mut entry.value),
            None => None,
        }
    }

    /// 设置键值并可指定过期时间（单位：毫秒）
    /// Set the key-value pair with an optional expiration time (in milliseconds).
    pub fn set(&mut self, key: &str, value: DbType, expiration_ms: Option<u64>) {
        let expiration_time = self.calculate_expiration(expiration_ms);

        let entry = DbEntry {
            value,
            expiration: expiration_time,
        };

        // 传播到 AOF
        self.propagate_aof_if_needed(key, &entry);

        // 存储数据
        self.storage.insert(key.to_string(), entry);
    }

    /// 设置键值并不传播到 AOF
    /// Set the key-value pair without propagating to AOF.
    pub fn set_without_aof(&mut self, key: &str, value: DbType, expiration_ms: Option<u64>) {
        let expiration_time = self.calculate_expiration(expiration_ms);

        let entry = DbEntry {
            value,
            expiration: expiration_time,
        };

        // 存储数据
        self.storage.insert(key.to_string(), entry);
    }

    /// 计算过期时间戳
    /// Calculate expiration timestamp in milliseconds.
    fn calculate_expiration(&self, expiration_ms: Option<u64>) -> Option<u64> {
        expiration_ms.map(|ms| {
            let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
            now.as_millis() as u64 + ms
        })
    }

    /// 传播到 AOF，如果需要的话
    /// Propagate to AOF if needed.
    fn propagate_aof_if_needed(&self, key: &str, entry: &DbEntry) {
        match &entry.value {
            DbType::String(value) => {
                let args = vec![key.to_string(), value.to_string()];
                let args_with_expiration = entry.expiration.map(|exp| {
                    let mut args = args.clone();
                    args.push(exp.to_string());
                    args
                });

                // 传播 AOF：有过期时间时带上过期时间
                if let Some(args_with_exp) = args_with_expiration {
                    propagate_aof("set".to_string(), args_with_exp);
                } else {
                    propagate_aof("set".to_string(), args);
                }
            }
            _ => {
                // 其他类型在各自command里面处理
            }
        }
    }

    /// 获取键值，如果已过期则返回 None、惰性删除（Lazy Deletion）
    /// Get the value for a key. If it is expired, return None and lazily delete it.
    pub fn get(&mut self, key: &str) -> Option<&DbType> {
        let expired = self.storage.get(key)
            .map_or(false, |entry| self.is_expired(entry));
        if expired {
            self.storage.remove(key);
            return None;
        }
        self.storage.get(key).map(|entry| &entry.value)
    }

    /// 删除键值
    /// Delete the key-value pair.
    pub fn del(&mut self, key: &str) -> bool {
        if !self.exists(key) {
            return false;
        }
        self.storage.remove(key).is_some()
    }

    /// 检查键值是否存在
    /// Check if the key exists.
    pub fn exists(&mut self, key: &str) -> bool {
        match self.storage.get(key) {
            Some(entry) if !self.is_expired(entry) => true,
            Some(_) => {
                self.storage.remove(key);
                false
            }
            None => false,
        }
    }

    /// 检查键值是否过期
    /// Check if the key-value entry is expired.
    fn is_expired(&self, entry: &DbEntry) -> bool {
        if let Some(expiration) = entry.expiration {
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64; // 毫秒时间戳
            // Millisecond timestamp
            now >= expiration
        } else {
            false
        }
    }

    /// 订阅频道
    /// Subscribe to a channel.
    pub fn subscribe(&mut self, channel: &str) -> &mut broadcast::Sender<Bytes> {
        self.pub_sub.entry(channel.to_string())
            .or_insert_with(|| {
                let (sender, _) = broadcast::channel(1024);
                sender
            })
    }

    /// Subscribe to a channel with wildcard support.
    /// This function checks if the channel name ends with a wildcard character (`*`).
    /// If it does, the subscription is handled under the `psubscribe` pattern, allowing wildcard matching.
    /// Otherwise, the subscription behaves like a regular `subscribe` to the specific channel.
    ///
    /// 订阅频道，允许使用通配符。
    /// 该函数会检查频道名称是否以通配符字符（`*`）结尾。如果是，它会按照 `psubscribe` 模式处理订阅，允许通配符匹配。
    /// 否则，订阅将像普通的 `subscribe` 一样处理，针对指定的频道进行订阅。
    pub fn psubscribe(&mut self, mut channel: &str) -> &mut broadcast::Sender<Bytes> {
        if channel.ends_with("*") {
            channel = &channel[..channel.len() - 1];
            self.psubscribes.entry(channel.to_string())
                .or_insert_with(|| {
                    let (sender, _) = broadcast::channel(1024);
                    sender
                })
        } else {
            self.pub_sub.entry(channel.to_string())
                .or_insert_with(|| {
                    let (sender, _) = broadcast::channel(1024);
                    sender
                })
        }
    }


    /// Publish a message to the specified channel.
    /// Returns the total number of subscribers who received the message (from both exact and wildcard matches).
    /// 向指定频道中发送消息。返回接收到消息的订阅者数量（包括精确匹配和通配符匹配的订阅者）。
    pub fn publish(&mut self, channel: &str, message: String) -> usize {
        let mut received_count = 0;

        // Handle psubscribe with wildcard matching
        // 处理 psubscribe 的通配符匹配
        for (pattern, sender) in self.psubscribes.iter_mut() {
            if channel.starts_with(pattern) {  // Check if the channel starts with the pattern
                sender.send(Bytes::from(message.clone())).unwrap_or(0);
                received_count += 1;  // Count the subscriber
            }
        }

        // Handle exact channel matching in pub_sub
        // 处理 pub_sub 中的精确频道匹配
        self.pub_sub.get(channel).map(|sender| {
            sender.send(Bytes::from(message)).unwrap_or(0);
            received_count += 1;  // Count the subscriber
        });

        received_count
    }
}

/// 定期删除（Active Expiration）
/// Active expiration: a task to periodically clean up expired keys.
async fn periodic_cleanup(mut db: Db, interval: Duration) {
    loop {
        cleanup_expired(&mut db);
        tokio::time::sleep(interval).await;
    }
}

/// 清理过期的数据
/// Cleanup expired data.
pub fn cleanup_expired(db: &mut Db) {
    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
    db.storage.retain(|_, entry| {
        match entry.expiration {
            // Keep the non-expired entries.
            Some(expiration) => expiration > now, // 保留未过期的条目
            // Entries without expiration always remain.
            None => true, // 没有过期时间的条目始终保留
        }
    });
}
