use std::fs::{OpenOptions, File};
use std::io::{Write, BufWriter, BufReader, BufRead};
use std::sync::{Arc};
use std::sync::{Mutex};
use lazy_static::lazy_static;
use crate::db::{Db, DbType};

lazy_static! {
    static ref AOF_WRITER: Arc<Mutex<AofWriter>> = Arc::new(Mutex::new(AofWriter::new("test.aof").expect("Failed to create AOF Writer")));
}

/// A simple AOF writer in Rust
#[derive(Debug,Clone)]
pub struct AofWriter {
    file: Arc<Mutex<BufWriter<File>>>,
    buffer: Arc<Mutex<Vec<u8>>>,
}

impl AofWriter {
    /// Create a new AOFWriter instance
    pub fn new(aof_file_path: &str) -> Result<AofWriter, std::io::Error> {
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .write(true)
            .open(aof_file_path)?;

        let file = Arc::new(Mutex::new(BufWriter::new(file)));
        let buffer = Arc::new(Mutex::new(Vec::new()));

        let aof_writer = AofWriter { file, buffer };

        // 启动一个异步任务，用于定期刷新缓冲区到磁盘
        tokio::spawn(
            periodic_flush(aof_writer.clone())
        );

        Ok(aof_writer)
    }

    /// Write a command to the AOF buffer in Redis format
    pub fn write_command(&self, command: &str, args: &[&str]) {
        let mut buf = Vec::new();

        // Append the command's arguments count in the AOF format (*<arg_count>\r\n)
        let args_count = args.len() + 1; // Including the command itself
        buf.push(b'*');
        buf.extend_from_slice(&args_count.to_string().into_bytes());
        buf.push(b'\r');
        buf.push(b'\n');

        // Write the command itself, prefixed with its length
        self.append_argument(&mut buf, command);

        // Write each argument with its length
        for arg in args {
            self.append_argument(&mut buf, arg);
        }

        // Append the command to the buffer
        let mut buffer = self.buffer.lock().unwrap();
        buffer.extend(buf);
    }

    /// Append an argument to the buffer with its length
    fn append_argument(&self, buf: &mut Vec<u8>, arg: &str) {
        buf.push(b'$');
        buf.extend_from_slice(&arg.len().to_string().into_bytes());
        buf.push(b'\r');
        buf.push(b'\n');
        buf.extend_from_slice(arg.as_bytes());
        buf.push(b'\r');
        buf.push(b'\n');
    }
}

pub fn propagate_aof(command: String, args: Vec<String>) {
    let writer = AOF_WRITER.clone(); // 克隆 Arc 指针

    // 在同步函数中启动异步任务
    let writer = writer.lock().unwrap(); // 获取锁
    let args_ref: Vec<&str> = args.iter().map(|s| s.as_str()).collect();

    // 确保异步调用 write_command
    writer.write_command(&command, &args_ref);
}


/// Flush the AOF buffer to the disk (Windows version)
pub fn flush(aof: &mut AofWriter) -> Result<(), std::io::Error> {
    let mut buffer = aof.buffer.lock().unwrap();
    if buffer.is_empty() {
        return Ok(());
    }

    let mut file = aof.file.lock().unwrap();
    file.write_all(&buffer)?;
    file.flush()?;
    buffer.clear();

    Ok(())
}

/// Flush the AOF buffer periodically based on time or external trigger
pub async fn periodic_flush(mut aof: AofWriter) {
    loop {
        // Simulate waiting for the next flush cycle (in reality, use a timer)
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        if let Err(e) =flush(&mut aof) {
            eprintln!("Error flushing AOF file: {}", e);
        }
    }
}

pub async fn load_aof(db: &mut Arc<Mutex<Db>>, aof_file_path: &str) -> Result<(), std::io::Error> {
    let file = File::open(aof_file_path)?;
    let reader = BufReader::new(file);

    let mut buffer = Vec::new();
    let mut command: Option<String> = None;
    let mut args: Vec<String> = Vec::new();

    // 迭代读取 AOF 文件的每一行
    for line in reader.lines() {
        let line = line?;
        if line.is_empty() {
            continue;
        }
        // 一行一行载入
        let  bytes = line.clone().as_bytes().to_vec(); // 使用 `to_vec` 创建一个新的 Vec<u8>，它是一个独立的堆分配的变量
        buffer.push(bytes);
        // 如果有完整的命令，则解析
        if let Some(cmd) = parse_aof_command(&mut buffer) {
            command = Some(cmd.0);
            args = cmd.1;
            apply_command_to_db(db, &command.unwrap(), &args)?;  // 将命令应用到数据库
            buffer.clear();  // 清空缓冲区以解析下一个命令
        }
    }

    Ok(())
}

fn parse_aof_command(buffer: &mut Vec<Vec<u8>>) -> Option<(String, Vec<String>)> {
    //  检查并解析参数个数
    // 获取第一个元素（参数个数标记），如果不存在则返回None
    let arg_count = match buffer.first()? {
        // 检查是否符合AOF格式：非空且以'*'开头
        arg_count_str if !arg_count_str.is_empty() && arg_count_str[0] == b'*' => {
            // 将'*'后面的部分转为字符串并解析为usize
            String::from_utf8_lossy(&arg_count_str[1..]).parse().ok()?
        }
        // 不符合格式要求则返回None
        _ => return None,
    };

    //  验证缓冲区长度是否匹配
    // AOF格式中参数个数与缓冲区长度关系应为：参数个数*2 + 1
    if buffer.len() != arg_count * 2 + 1 {
        return None;
    }

    //  移除参数个数标记（'*数字'部分）
    buffer.remove(0);

    //  处理命令和参数
    let mut args = Vec::with_capacity(arg_count); // 预分配空间
    for _ in 0..arg_count {
        // 跳过长度指示符（AOF中我们假设它是正确的）
        buffer.remove(0);
        // 获取实际参数值并转为String
        args.push(String::from_utf8_lossy(buffer.first()?).into_owned());
        // 移除已处理的参数
        buffer.remove(0);
    }

    // 分割出命令和参数
    if args.is_empty() {
        None
    } else {
        let command = args.remove(0); // 第一个元素是命令
        Some((command, args)) // 返回命令和剩余参数
    }
}


fn apply_command_to_db(
    db: &mut Arc<Mutex<Db>>,
    command: &str,
    args: &[String],
) -> Result<(), std::io::Error> {
    let mut db = db.lock().unwrap();


    match command.to_uppercase().as_str() {
        "SET" => {
            if args.len() < 2 {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "SET command expects at least 2 arguments",
                ));
            }
            let key = &args[0];
            let value = &args[1];
            db.set(key, DbType::String(value.clone()), None);
        }
        "APPEND" => {
            if args.len() < 2 {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "APPEND command expects at least 2 arguments",
                ));
            }
            let key = &args[0];
            let value = &args[1];
            let current_value = db.get(key);

            let new_value = match current_value {
                Some(DbType::String(existing_value)) => {
                    format!("{}{}", existing_value, value)
                }
                _ => value.clone(),
            };
            db.set(key, DbType::String(new_value), None);
        }
        // Handle other commands here as needed...
        _ => {
            eprintln!("Unrecognized command: {}", command);
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Unrecognized command",
            ));
        }
    }

    Ok(())
}