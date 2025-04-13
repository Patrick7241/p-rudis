#[derive(Debug)]
pub struct Shutdown {
    /// 是否接收到关闭信号
    is_shutdown:bool
}

impl Shutdown{
    /// TODO 待实现
    pub fn new()->Self{
        Shutdown{
            is_shutdown:false
        }
    }
    pub fn is_shutdown(&self)->bool{
        self.is_shutdown
    }

}
