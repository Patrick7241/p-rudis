use crate::connection::ConnectionHandler;
use crate::db::Db;
use crate::parse::Parse;

pub struct Get{
    key: String
}
impl Get{
    pub fn get_command(
        db:Db,
        connection:ConnectionHandler,
        parse: Parse) -> crate::Result<()> {
        println!("成功被调用 get");
        Ok(())
    }
}