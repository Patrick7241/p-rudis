use std::sync::{Arc, Mutex};
use crate::db::Db;
use crate::frame::Frame;
use crate::parse::Parse;

/// ping -> pong :)

pub struct Ping;

impl Ping {
    pub fn ping_command(_db: &mut Arc<Mutex<Db>>, _parse: &mut Parse) -> crate::Result<Frame> {
        Ok(Frame::Simple("PONG".to_string()))
    }
}