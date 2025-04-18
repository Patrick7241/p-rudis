use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use std::sync::Mutex;

#[derive(Debug)]
pub struct DbHolder {
    db: Arc<Mutex<Db>>,
}

#[derive(Clone, Debug)]
pub struct Db {
    storage: HashMap<String, DbEntry>,
}

#[derive(Clone, Debug)]
pub struct DbEntry {
    value: DbType,
    expiration: Option<u64>,  // 存储过期时间，单位 毫秒
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
        };
        // 开启定时任务，定时处理过期的键值
        tokio::spawn(periodic_cleanup(db.clone(), Duration::from_secs(10)));
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

