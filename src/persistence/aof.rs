use std::collections::{HashMap, VecDeque};
use std::fs::{OpenOptions, File};
use std::io::{Write, BufWriter, BufReader, BufRead};
use std::sync::{Arc, Mutex};
use std::thread::sleep;
use std::time::Instant;
use lazy_static::lazy_static;
use log::{info, error};
use crate::config::get_aof_config;
use crate::db::{Db, DbType};
use crate::persistence::aof_command::{handle_del_command, handle_hdel_command, handle_hset_command, handle_lpop_command, handle_lpush_command, handle_lrem_command, handle_lset_command, handle_rpop_command, handle_rpush_command, handle_set_command};

lazy_static! {
    static ref AOF_WRITER: Arc<Mutex<AofWriter>> = {
        let aof_config = get_aof_config();
        Arc::new(Mutex::new(AofWriter::new(aof_config.file_path.as_str()).unwrap_or_else(|e| {
            error!("Failed to create AOF Writer: {}", e);
            std::process::exit(1);
        })))
    };
}

#[derive(Debug, Clone)]
pub struct AofWriter {
    file: Arc<Mutex<BufWriter<File>>>,
    buffer: Arc<Mutex<Vec<u8>>>,
}

impl AofWriter {
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
        tokio::spawn(periodic_flush(aof_writer.clone()));

        Ok(aof_writer)
    }

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
    let aof_config=get_aof_config();
    if !aof_config.enabled {
        return;
    }
    let writer = AOF_WRITER.clone(); // 克隆 Arc 指针

    let writer = writer.lock().unwrap(); // 获取锁
    let args_ref: Vec<&str> = args.iter().map(|s| s.as_str()).collect();

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
    let aof_config=get_aof_config();
    loop {
        tokio::time::sleep(std::time::Duration::from_secs(aof_config.appendfsync)).await;
        if let Err(e) = flush(&mut aof) {
            error!("Error flushing AOF file: {}", e);
        }
    }
}

pub async fn load_aof(db: &mut Arc<Mutex<Db>>, aof_file_path: &str) -> Result<(u128, ()), std::io::Error> {
    // Start timing
    let start_time = Instant::now();

    let file = File::open(aof_file_path)?;
    let reader = BufReader::new(file);

    let mut buffer = Vec::new();
    let mut command: Option<String> = None;
    let mut args: Vec<String> = Vec::new();

    // Iterate through each line in the AOF file
    for line in reader.lines() {
        let line = line?;
        if line.is_empty() {
            continue;
        }

        let bytes = line.clone().as_bytes().to_vec();
        buffer.push(bytes);

        // If a complete command is found, parse and apply it
        if let Some(cmd) = parse_aof_command(&mut buffer) {
            command = Some(cmd.0);
            args = cmd.1;

            if let Err(e) = apply_command_to_db(db, &command.unwrap(), &args) {
                error!("Failed to apply command: {}", e);
            }

            buffer.clear();
        }
    }

    // Measure the time taken
    let duration = start_time.elapsed();

    Ok((duration.as_millis(), ())) // Return the time in milliseconds
}

fn parse_aof_command(buffer: &mut Vec<Vec<u8>>) -> Option<(String, Vec<String>)> {
    let arg_count = match buffer.first() {
        Some(arg_count_str) if !arg_count_str.is_empty() && arg_count_str[0] == b'*' => {
            String::from_utf8_lossy(&arg_count_str[1..]).parse().ok()?
        }
        _ => return None,
    };

    if buffer.len() != arg_count * 2 + 1 {
        return None;
    }

    buffer.remove(0);
    let mut args = Vec::with_capacity(arg_count);
    for _ in 0..arg_count {
        buffer.remove(0);
        if let Some(arg) = buffer.first() {
            args.push(String::from_utf8_lossy(arg).into_owned());
        }
        buffer.remove(0);
    }

    if args.is_empty() {
        None
    } else {
        let command = args.remove(0);
        Some((command, args))
    }
}

fn apply_command_to_db(
    db: &mut Arc<Mutex<Db>>,
    command: &str,
    args: &[String],
) -> Result<(), std::io::Error> {
    let mut db = db.lock().unwrap();

    match command.to_lowercase().as_str() {
        "set" => handle_set_command(&mut db, args)?,
        "del" => handle_del_command(&mut db, args)?,
        "hset" => handle_hset_command(&mut db, args)?,
        "hdel" => handle_hdel_command(&mut db, args)?,
        "lpush" => handle_lpush_command(&mut db, args)?,
        "rpush" => handle_rpush_command(&mut db, args)?,
        "lpop" => handle_lpop_command(&mut db, args)?,
        "rpop" => handle_rpop_command(&mut db, args)?,
        "lset" => handle_lset_command(&mut db, args)?,
        "lrem" => handle_lrem_command(&mut db, args)?,
        _ => info!("Unsupported command: {}", command),
    }
    Ok(())
}
