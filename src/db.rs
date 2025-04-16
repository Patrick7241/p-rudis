use std::collections::{HashMap, HashSet, VecDeque};

#[derive(Debug)]
pub struct DbHolder{
    db:Db
}
#[derive(Clone,Debug)]
pub struct Db{
    storage: HashMap<String,DbType>
}

/// 数据结构类型
#[derive(Clone,Debug)]
enum DbType{
    /// 字符串
    String(String),
    /// 哈希
    Hash(HashMap<String,String>),
    /// 列表
    List(VecDeque<String>),
    /// 集合
    Set(HashSet<String>),
    /// 有序集合 TODO collections没有，自己写，待完善
    ZSet(String),
    /// 位图 TODO collections没有，自己写，待完善
    BitMap(String)
}


impl DbHolder{
    /// 初始化数据库
    pub fn new()->Self{
        DbHolder{
            db:Db::new()
        }
    }
    pub fn clone(&self)->Db{
        self.db.clone()
    }
}

impl Db{
    /// 初始化数据库，并启动定时任务，删除过期的键
    fn new()->Db{
        let db=Db{
            storage:HashMap::new()
        };
        // TODO 启动定时任务，删除过期的键
        db
    }

}