use tokio::sync::broadcast;

#[derive(Debug)]
pub struct Shutdown {
    /// 是否接收到关闭信号
    /// Whether the shutdown signal has been received
    is_shutdown: bool,
    /// 接收发布订阅模式的关闭信号
    /// Receiver for the shutdown signal in the publish-subscribe model
    notify: broadcast::Receiver<()>,
}

impl Shutdown {
    /// TODO 待实现
    /// TODO: To be implemented
    pub fn new(notify: broadcast::Receiver<()>) -> Self {
        Shutdown {
            is_shutdown: false,
            notify,
        }
    }

    /// 判断是否接收到关闭信号
    /// Check whether the shutdown signal has been received
    pub fn is_shutdown(&self) -> bool {
        self.is_shutdown
    }

    /// 接收关闭信号
    /// Receive the shutdown signal
    pub async fn recv(&mut self) {
        // 如果本来就接收到关闭信号，则直接返回
        // If the shutdown signal has already been received, return immediately
        if self.is_shutdown {
            return;
        }
        let _ = self.notify.recv().await;
        self.is_shutdown = true;  // Mark as shutdown after receiving the signal
    }

    /// 设置关闭信号
    /// Set the shutdown signal
    pub fn trigger(&mut self) {
        self.is_shutdown = true;
    }
}
