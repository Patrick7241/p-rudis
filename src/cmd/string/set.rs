use crate::db::Db;
use crate::frame::Frame;
use crate::parse::Parse;

/// string类型 set命令
/// 支持 EX PX 设置过期时间，EX单位是秒，PX单位是毫秒
/// 支持 NX XX NX表示只有当键不存在时才设置，XX表示只有当键存在时才设置

pub struct Set{
    key:String,
    value:String,
    expiration:Option<u64>,
    nx:bool,
    xx:bool,
}
impl Set {
    pub fn set_command(
        db: &mut Db,
        parse: &mut Parse,
    ) -> crate::Result<Frame> {
        Ok(Frame::Null)
    }
    // fn parse_command(parse: &mut Parse) -> crate::Result<Self> {
    //     let key = parse.next_string()?.to_lowercase();
    //     if key.is_empty() {
    //         return Err(Box::new(Error::new(
    //             std::io::ErrorKind::InvalidInput,
    //             "ERR wrong number of arguments for 'set' command",
    //         )));
    //     }
    //
    //     Ok(Set {
    //         key,
    //         value: "".to_string(),
    //         expiration: None,
    //         nx: false,
    //     })
    // }
}