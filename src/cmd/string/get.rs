use std::any::Any;

pub struct Get{
    key: String
}
impl Get{
    pub fn get_command(a: Option<Box<dyn Any>>) -> Option<Box<dyn Any>> {
        match a {
            Some(value) => {
                Some(Box::new("Hello, world!".to_string()))
            }
            None => {
                // 如果没有传递参数，返回一个默认的响应
                println!("No input provided.");
                Some(Box::new("Default response".to_string()))
            }
        }
    }
}