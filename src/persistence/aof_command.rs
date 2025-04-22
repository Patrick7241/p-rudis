use crate::db::{Db, DbType};
use std::io::{Error, ErrorKind};

pub fn handle_set_command(
    db: &mut Db,
    args: &[String],
) -> Result<(), Error> {
    if args.len() < 2 {
        return Err(Error::new(
            ErrorKind::InvalidData,
            "SET command expects at least 2 arguments",
        ));
    }
    let key = &args[0];
    let value = &args[1];
    let ttl = args.get(2).map(|ttl_str| ttl_str.parse::<u64>().ok()).flatten();

    if let Some(ttl_value) = ttl {
        db.set_without_aof(key, DbType::String(value.clone()), Some(ttl_value));
    } else {
        db.set_without_aof(key, DbType::String(value.clone()), None);
    }
    Ok(())
}

pub fn handle_del_command(
    db: &mut Db,
    args: &[String],
) -> Result<(), Error> {
    if args.len() < 1 {
        return Err(Error::new(
            ErrorKind::InvalidData,
            "DEL command expects at least 1 argument",
        ));
    }
    db.del(&args[0]);
    Ok(())
}

pub fn handle_hset_command(
    db: &mut Db,
    args: &[String],
) -> Result<(), Error> {
    if args.len() < 3 {
        return Err(Error::new(
            ErrorKind::InvalidData,
            "HSET command expects at least 3 arguments",
        ));
    }
    let key = &args[0];
    let field = &args[1];
    let value = &args[2];
    if db.get_dbtype_mut(key).is_some() {
        match db.get_dbtype_mut(key) {
            Some(DbType::Hash(hash)) => {
                hash.insert(field.to_string(), value.to_string());
            },
            _ => return Err(Error::new(
                ErrorKind::InvalidData,
                "Key exists but is not a hash table",
            )),
        }
    } else {
        db.set_without_aof(key, DbType::Hash(std::collections::HashMap::from([(field.to_string(), value.to_string())])), None);
    }
    Ok(())
}

pub fn handle_hdel_command(
    db: &mut Db,
    args: &[String],
) -> Result<(), Error> {
    if args.len() < 2 {
        return Err(Error::new(
            ErrorKind::InvalidData,
            "HDEL command expects at least 2 arguments",
        ));
    }
    if let Some(DbType::Hash(hash)) = db.get_dbtype_mut(&args[0]) {
        hash.remove(&args[1]);
    }
    Ok(())
}

pub fn handle_lpush_command(
    db: &mut Db,
    args: &[String],
) -> Result<(), Error> {
    if args.len() < 2 {
        return Err(Error::new(
            ErrorKind::InvalidData,
            "LPUSH command expects at least 2 arguments",
        ));
    }

    if let Some(DbType::List(list)) = db.get_dbtype_mut(&args[0]) {
        for value in args[1..].iter() {
            list.push_front(value.to_string());
        }
    } else {
        db.set_without_aof(&args[0], DbType::List(args[1..].iter().rev().map(|s| s.to_string()).collect()), None);
    }
    Ok(())
}

pub fn handle_rpush_command(
    db: &mut Db,
    args: &[String],
) -> Result<(), Error> {
    if args.len() < 2 {
        return Err(Error::new(
            ErrorKind::InvalidData,
            "RPUSH command expects at least 2 arguments",
        ));
    }

    if let Some(DbType::List(list)) = db.get_dbtype_mut(&args[0]) {
        for value in args[1..].iter() {
            list.push_back(value.to_string());
        }
    } else {
        db.set_without_aof(&args[0], DbType::List(args[1..].iter().map(|s| s.to_string()).collect()), None);
    }
    Ok(())
}

pub fn handle_lpop_command(
    db: &mut Db,
    args: &[String],
) -> Result<(), Error> {
    if args.len() < 1 {
        return Err(Error::new(
            ErrorKind::InvalidData,
            "LPOP command expects at least 1 argument",
        ));
    }
    if let Some(DbType::List(list)) = db.get_dbtype_mut(&args[0]) {
        list.pop_front();
    }
    Ok(())
}

pub fn handle_rpop_command(
    db: &mut Db,
    args: &[String],
) -> Result<(), Error> {
    if args.len() < 1 {
        return Err(Error::new(
            ErrorKind::InvalidData,
            "RPOP command expects at least 1 argument",
        ));
    }
    if let Some(DbType::List(list)) = db.get_dbtype_mut(&args[0]) {
        list.pop_back();
    }
    Ok(())
}

pub fn handle_lset_command(
    db: &mut Db,
    args: &[String],
) -> Result<(), Error> {
    if args.len() < 3 {
        return Err(Error::new(
            ErrorKind::InvalidData,
            "LSET command expects at least 3 arguments",
        ));
    }
    if let Some(DbType::List(list)) = db.get_dbtype_mut(&args[0]) {
        let index = args[1].parse::<usize>().unwrap();
        if index < list.len() {
            list[index] = args[2].to_string();
        }
    }
    Ok(())
}

pub fn handle_lrem_command(
    db: &mut Db,
    args: &[String],
) -> Result<(), Error> {
    if args.len() < 3 {
        return Err(Error::new(
            ErrorKind::InvalidData,
            "LREM command expects at least 3 arguments",
        ));
    }
    if let Some(DbType::List(list)) = db.get_dbtype_mut(&args[0]) {
        let count = args[1].parse::<i32>().unwrap();
        if count == 0 {
            list.retain(|x| x != &args[2]);
        } else if count > 0 {
            for _ in 0..count {
                list.remove((&args[2]).parse().unwrap());
            }
        } else {
            for _ in 0..-count {
                list.remove((&args[2]).parse().unwrap());
            }
        }
    }
    Ok(())
}
