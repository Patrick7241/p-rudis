use std::sync::{Arc, Mutex};
use crate::db::Db;
use crate::frame::Frame;
use crate::parse::Parse;

/// echo message -> message :)

pub struct Echo{
    message:String
}

impl Echo{
    pub fn echo_command(_db: &mut Arc<Mutex<Db>>, parse: &mut Parse) -> crate::Result<Frame> {
        match Echo::parse_command(parse) {
            Ok(echo) => {
                Ok(Frame::Bulk(echo.message.into_bytes()))
            }
            Err(_) => {
                Ok(Frame::Error("ERR wrong number of arguments for 'echo' command".to_string()))
            }
        }
    }

    fn parse_command(parse: &mut Parse) -> crate::Result<Self>{
        let message = parse.next_string()?;
        Ok(Echo{message})
    }

}

