use std::collections::{HashMap, HashSet, VecDeque};
use std::pin::Pin;
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use std::sync::Mutex;
use tokio::sync::broadcast;
use tokio_stream::{Stream, StreamExt, StreamMap};
use bytes::Bytes;

pub(crate) type Messages = Pin<Box<dyn Stream<Item = Bytes> + Send>>;

#[derive(Debug)]
pub struct DbHolder {
    db: Arc<Mutex<Db>>,
}

#[derive(Clone, Debug)]
pub struct Db {
    storage: HashMap<String, DbEntry>,
    /// 发布/订阅模式
    /// 键是发布/订阅模式下的频道，值是每个频道的广播发送器
    pub_sub: HashMap<String, broadcast::Sender<Bytes>>
}

#[derive(Clone, Debug)]
pub struct DbEntry {
    /// 基本数据结构的数据类型
    value: DbType,
    /// 存储过期时间，单位 毫秒
    expiration: Option<u64>,
}

#[derive(Clone, Debug)]
pub enum DbType {
    String(String),
    Hash(HashMap<String, String>),
    List(VecDeque<String>),
    Set(HashSet<String>),
    ZSet(String),  // 有序集合
    BitMap(String), // 位图
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
        let db=Db {
            storage: HashMap::new(),
            pub_sub: HashMap::new(),
        };
        // 开启定时任务，定时处理过期的键值
        tokio::spawn(periodic_cleanup(db.clone(), Duration::from_secs(1024)));
        db
    }

    /// 获取DbType的可变引用
    pub fn get_dbtype_mut(&mut self,key:&str)->Option<&mut DbType>{
        match self.storage.get_mut(key){
            Some(entry)=>Some(&mut entry.value),
            None=>None
        }
    }

    /// 设置键值并可指定过期时间（单位：毫秒）
    pub fn set(&mut self, key: &str, value: DbType, expiration_ms: Option<u64>) {
        let expiration_time = expiration_ms.map(|ms| {
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap();
            // 直接计算毫秒级过期时间戳
            now.as_millis() as u64 + ms
        });
        let entry = DbEntry {
            value,
            expiration: expiration_time,
        };

        self.storage.insert(key.to_string(), entry);
    }

    /// 获取键值，如果已过期则返回 None、
    /// 惰性删除（Lazy Deletion）
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
    pub fn del(&mut self, key: &str) -> bool {
        if !self.exists(key){
            return false;
        }
        self.storage.remove(key).is_some()
    }

    /// 检查键值是否存在
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
    fn is_expired(&self, entry: &DbEntry) -> bool {
        if let Some(expiration) = entry.expiration {
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64; // 毫秒时间戳
            now >= expiration
        } else {
            false
        }
    }

    pub fn subscribe(&mut self, channel: &str) -> &mut broadcast::Sender<Bytes> {
        self.pub_sub.entry(channel.to_string())
            .or_insert_with(|| {
                let (sender, _) = broadcast::channel(1024);
                sender
            })
        // if self.pub_sub.contains_key(channel) {
        //     println!("存在： {}",channel);
        //     self.pub_sub.get_mut(channel).unwrap()
        // } else {
        //     println!("不存在,创建： {}",channel);
        //     let (sender, _) = broadcast::channel(1024);
        //     self.pub_sub.insert(channel.to_string(), sender);
        //     self.pub_sub.get_mut(channel).unwrap()
        // }
    }

    /// 向指定频道中发送消息
    /// 返回接收到消息的订阅者数量
    pub fn publish(&mut self, channel: &str, message: String) -> usize {
        self.pub_sub.get(channel).map(|sender| {
            sender.send(Bytes::from(message)).unwrap_or(0)
        }).unwrap_or(0)
        // if let Some(sender) = self.pub_sub.get(channel) {
        //     if sender.send(Bytes::from(message)).is_ok() {
        //         println!("Sent message to channel {}", channel);
        //         sender.receiver_count()
        //     } else {
        //         println!("Failed to send message to channel {}", channel);
        //         0
        //     }
        // } else {
        //     println!("Channel {} not found", channel);
        //     0
        // }
    }

}

/// 定期删除（Active Expiration）
async fn periodic_cleanup(mut db: Db, interval: Duration) {
    // TODO 待添加，停止条件
    loop {
       cleanup_expired(&mut db);
        tokio::time::sleep(interval).await;
    }
}
/// 清理过期的数据
pub fn cleanup_expired(db:&mut Db) {
    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
    db.storage.retain(|_, entry| {
        match entry.expiration {
            Some(expiration) => expiration > now, // 保留未过期的条目
            None => true, // 没有过期时间的条目始终保留
        }
    });
}

