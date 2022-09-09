//! 对外暴露的接口

use tokio::sync::mpsc;

use crate::{message::Message, raft::Raft};

/// 可以多次克隆，在各处使用
#[derive(Clone)]
pub struct Client {
    incoming_tx: mpsc::Sender<Message>,
}

impl Client {
    pub fn new() -> (Self, Raft) {
        todo!()
    }

    /// 全局计时器
    pub fn tick(&self) {}

    /// 来自客户端和对等节点的命令
    pub async fn request(&self, _msg: Message) {}
}
