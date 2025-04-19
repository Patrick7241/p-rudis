use tokio::sync::broadcast;

#[derive(Debug)]
pub struct Shutdown {
    /// 是否接收到关闭信号
    is_shutdown:bool,
    /// 接收发布订阅模式的关闭信号
    notify:broadcast::Receiver<()>,
}

impl Shutdown{
    /// TODO 待实现
    pub fn new(notify:broadcast::Receiver<()>)->Self{
        Shutdown{
            is_shutdown:false,
            notify
        }
    }
    /// 判断是否接收到关闭信号
    pub fn is_shutdown(&self)->bool{
        self.is_shutdown
    }

    pub async fn recv(&mut self){
        // 如果本来就接收到关闭信号，则直接返回
        if self.is_shutdown{
            return
        }
        let _=self.notify.recv().await;
        self.is_shutdown=true;
    }

    /// 设置关闭信号
    pub fn trigger(&mut self){
        self.is_shutdown=true
    }

}
