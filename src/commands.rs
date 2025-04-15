use std::any::Any;
use p_rudis::cmd;
/// 定义命令元数据，后续命令都可以添加到这里
pub static COMMANDS: &[(&str, &str, &str, fn(Option<Box<dyn Any>>) -> Option<Box<dyn Any>>)] = &[
    ("get", "返回指定键的字符串值。", "O(1)", cmd::string::get::Get::get_command),
];



// pub enum Command {
// }
// 
// impl Command {
//     /// 解析命令到具体指令类型
//     pub fn get_command(frame: Frame){
//         match Self::parse_command_from_client(frame) {
//             Ok(frame) => println!("{:?}", frame),
//             Err(e) => eprintln!("Error: {}", e),
//         }
//     }
// 
//     /// 解析客户端发送的命令，根据RESP协议，客户端发送过来只能是Array
//     pub fn parse_command_from_client(frame: Frame)->p_rudis::Result<Vec<Frame>>{
//         match frame {
//             Frame::Array(data)=>{
//                 Ok(data)
//             }
//             frame=>{
//           Err(format!("期望 Array 类型,实际 {:?}，不符合 RESP 协议", frame).into())
//             }
//         }
//     }
// }