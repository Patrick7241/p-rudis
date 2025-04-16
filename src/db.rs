use std::collections::{HashMap, HashSet, VecDeque};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

#[derive(Debug)]
pub struct DbHolder {
    db: Db,
}

#[derive(Clone, Debug)]
pub struct Db {
    storage: HashMap<String, DbEntry>,
}

#[derive(Clone, Debug)]
pub struct DbEntry {
    value: DbType,
    expiration: Option<u64>,  // 存储过期时间，单位是秒
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
        DbHolder {
            db: Db::new(),
        }
    }

    pub fn clone(&self) -> Db {
        self.db.clone()
    }
}

impl Db {
    pub fn new() -> Db {
        let db=Db {
            storage: HashMap::new(),
        };
        // 开启定时任务，定时处理过期的键值
        tokio::spawn(periodic_cleanup(db.clone(), Duration::from_secs(10)));
        db
    }

    /// 设置键值并可指定过期时间（单位：秒）
    pub fn set(&mut self, key: &str, value: DbType, expiration: Option<u64>) {
        let expiration_time = expiration.map(|exp| {
            let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
            now.as_secs() + exp  // 设置过期时间
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

    /// 检查条目是否过期
    fn is_expired(&self, entry: &DbEntry) -> bool {
        if let Some(expiration) = entry.expiration {
            let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
            return now > expiration;
        }
        false
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

