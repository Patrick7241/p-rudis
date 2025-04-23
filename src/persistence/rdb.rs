use std::collections::{HashMap, VecDeque};
use crate::db::{Db, DbEntry, DbType};
use bytes::{Buf, BufMut, Bytes, BytesMut};
use std::fs::{File, OpenOptions};
use std::io;
use std::io::{BufWriter, Read, Seek, SeekFrom, Write};
use std::sync::{Arc, Mutex};
use std::time::{Instant, SystemTime, UNIX_EPOCH};


// Constants
mod constants {
    pub const RDB_MAGIC: &[u8] = b"REDIS";
    pub const RDB_VERSION: &[u8] = b"0001";

    // Data type identifiers
    pub const RDB_TYPE_STRING: u8 = 0;
    pub const RDB_TYPE_LIST: u8 = 1;
    pub const RDB_TYPE_SET: u8 = 2;
    pub const RDB_TYPE_ZSET: u8 = 3;
    pub const RDB_TYPE_HASH: u8 = 4;

    // Opcodes
    pub const RDB_OPCODE_AUX: u8 = 250;
    pub const RDB_OPCODE_EXPIRETIME_MS: u8 = 252;
    pub const RDB_OPCODE_SELECTDB: u8 = 254;
    pub const RDB_OPCODE_EOF: u8 = 255;
}

use constants::*;

#[derive(Debug, Clone)]
pub struct RdbWriter {
    file: Arc<Mutex<BufWriter<File>>>,
    buffer: BytesMut,
}

impl RdbWriter {
    pub fn new(rdb_file_path: &str) -> Self {
        let file = OpenOptions::new()
            .create(true)
            .write(true)
            .open(rdb_file_path)
            .expect("Failed to open RDB file");

        Self {
            file: Arc::new(Mutex::new(BufWriter::new(file))),
            buffer: BytesMut::new(),
        }
    }

    pub async fn load_file(rdb_file_path: &str) -> io::Result<Self> {
        let mut file = File::open(rdb_file_path)?;
        let mut buffer = BytesMut::with_capacity(10 * 1024 * 1024);
        let mut vec = buffer.to_vec();

        file.read_to_end(&mut vec)?;
        if vec.is_empty() {
            return Err(io::Error::new(io::ErrorKind::UnexpectedEof, "File is empty"));
        }

        buffer.put_slice(&vec);

        Ok(Self {
            file: Arc::new(Mutex::new(BufWriter::new(file))),
            buffer,
        })
    }

    // Header operations
    fn write_header(&mut self) {
        self.buffer.put_slice(RDB_MAGIC);
        self.buffer.put_slice(RDB_VERSION);
    }

    // Key-value operations
    pub fn save_key_value_pair(&mut self, key: &str, value: &DbEntry, now: u64) {
        if let Some(expire_time) = value.expiration {
            if now > expire_time {
                return;
            }
            self.write_expire_time(expire_time);
        }

        self.save_db_type(&value.value);
        self.save_string(key);
        self.save_value(&value.value);
    }

    fn write_expire_time(&mut self, expire_time: u64) {
        self.buffer.put_u8(RDB_OPCODE_EXPIRETIME_MS);
        self.buffer.put_u64(expire_time);
    }

    fn save_string(&mut self, s: &str) {
        self.buffer.put_u8(s.len() as u8);
        self.buffer.put_slice(s.as_bytes());
    }

    fn save_value(&mut self, value: &DbType) {
        match value {
            DbType::String(s) => self.save_string(s),
            DbType::List(list) => {
                self.buffer.put_u8(list.len() as u8);
                for item in list {
                    self.save_string(item);
                }
            }
            DbType::Hash(map) => {
                self.buffer.put_u8(map.len() as u8);
                for (key, value) in map {
                    self.save_string(key);
                    self.save_string(value);
                }
            }
        }
    }

    fn save_db_type(&mut self, db_type: &DbType) {
        let type_code = match db_type {
            DbType::String(_) => RDB_TYPE_STRING,
            DbType::List(_) => RDB_TYPE_LIST,
            DbType::Hash(_) => RDB_TYPE_HASH,
        };
        self.buffer.put_u8(type_code);
    }

    // Loading operations
    fn load_string_object(&mut self) -> io::Result<BytesMut> {
        let len = self.buffer.get_u8() as usize;
        Ok(self.buffer.split_to(len))
    }

    fn load_object(&mut self, obj_type: u8) -> io::Result<DbType> {
        match obj_type {
            RDB_TYPE_STRING => {
                let bytes = self.load_string_object()?.freeze();
                Ok(DbType::String(String::from_utf8(bytes.to_vec()).unwrap()))
            }
            RDB_TYPE_LIST => {
                let len = self.buffer.get_u8() as usize;
                let mut list = VecDeque::with_capacity(len);

                for _ in 0..len {
                    let bytes = self.load_string_object()?.freeze();
                    list.push_back(String::from_utf8(bytes.to_vec()).unwrap());
                }

                Ok(DbType::List(list))
            }
            RDB_TYPE_HASH => {
                let len = self.buffer.get_u8() as usize;
                let mut map = HashMap::with_capacity(len);

                for _ in 0..len {
                    let key = self.load_string_object()?.freeze();
                    let value = self.load_string_object()?.freeze();
                    map.insert(
                        String::from_utf8(key.to_vec()).unwrap(),
                        String::from_utf8(value.to_vec()).unwrap(),
                    );
                }

                Ok(DbType::Hash(map))
            }
            _ => panic!("Unsupported RDB type"),
        }
    }
}

// Public interface functions
pub async fn dump(db: &Arc<Mutex<Db>>, rdb_file_path: &str) -> RdbWriter {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64;

    let mut rdb = RdbWriter::new(rdb_file_path);
    rdb.write_header();

    // Select database 0
    rdb.buffer.put_u8(RDB_OPCODE_SELECTDB);
    rdb.buffer.put_u32(0);

    // Save all key-value pairs
    let db = db.lock().unwrap();
    for (key, value) in db.iter() {
        rdb.save_key_value_pair(key, value, now);
    }

    // Write EOF marker
    rdb.buffer.put_u8(RDB_OPCODE_EOF);
    rdb
}

pub fn save(db: Arc<Mutex<Db>>, rdb_file_path: String) -> io::Result<()> {
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(std::time::Duration::from_secs(5)).await;

            let mut rdb = dump(&db, &rdb_file_path).await;
            let mut file = rdb.file.lock().unwrap();

            file.get_ref().set_len(0).unwrap();
            file.write_all(&rdb.buffer).unwrap();
            file.flush().unwrap();

            rdb.buffer.clear();
        }
    });

    Ok(())
}

pub async fn load_rdb(db: &Arc<Mutex<Db>>, rdb: &mut RdbWriter) -> Result<(u128, ()), std::io::Error> {
    let start_time = Instant::now();
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64;

    // Verify magic number and version
    let magic = [RDB_MAGIC, RDB_VERSION].concat();
    if rdb.buffer.len() < magic.len() || &rdb.buffer[..magic.len()] != magic {
        return Err(io::Error::new(io::ErrorKind::InvalidData, "Invalid RDB file"));
    }
    rdb.buffer.advance(magic.len());

    // Process RDB contents
    loop {
        let mut expiration = None;
        let mut opcode = rdb.buffer.get_u8();

        match opcode {
            RDB_OPCODE_EXPIRETIME_MS => {
                expiration = Some(rdb.buffer.get_u64());
                opcode = rdb.buffer.get_u8();
            }
            RDB_OPCODE_SELECTDB => {
                let _db_index = rdb.buffer.get_u32();
                continue;
            }
            RDB_OPCODE_EOF => break,
            _ => (),
        }

        let key = rdb.load_string_object()?.freeze();
        let value = rdb.load_object(opcode)?;

        if let Some(exp) = expiration {
            if exp < now {
                continue;
            }
        }

        let mut db = db.lock().unwrap();
        let key_str = String::from_utf8(key.to_vec()).unwrap();

        match value {
            DbType::String(s) => {
                db.set(&key_str, DbType::String(s), None);
            }
            DbType::List(list) => {
                if let Some(DbType::List(existing)) = db.get_dbtype_mut(&key_str) {
                    existing.extend(list);
                } else {
                    db.set(&key_str, DbType::List(list), None);
                }
            }
            DbType::Hash(map) => {
                if let Some(DbType::Hash(existing)) = db.get_dbtype_mut(&key_str) {
                    existing.extend(map);
                } else {
                    db.set(&key_str, DbType::Hash(map), None);
                }
            }
        }
    }

    // Measure the time taken
    let duration = start_time.elapsed();

    Ok((duration.as_millis(), ())) // Return the time in milliseconds
}