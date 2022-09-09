//! 对外暴露的方法都在这

use std::collections::HashMap;

use tokio::sync::oneshot;

use crate::{message::Message, MessageId, NodeId};

#[derive(Debug, thiserror::Error)]
pub enum Error {}

/// client tx ----> rx raft 客户端交给raft处理的消息
/// node tx ----> rx raft 其它节点交给raft处理的消息
/// request/reply rx <---- tx raft raft交给当前应用处理的消息，包括消息的回复/发送给其它节点的消息/需要客户端对状态机进行操作的消息
pub struct Raft {
    /// 当前节点ID
    id: NodeId,
    /// RoleState 角色节点
    /// 同步请求的响应
    requests: HashMap<MessageId, oneshot::Receiver<Message>>,
}

impl Raft {
    pub fn new(id: NodeId) -> Self {
        Self {
            id,
            // Node
            requests: HashMap::new(),
        }
    }

    /// raft 主循环，每次调用此方法时，Raft 进度向前步进
    /// 1. 接收 client 通过 incoming_tx 传入的消息，进行处理
    /// 2. 数据流动：
    ///     * Raft --> Node 直接调用 Node 的方法
    ///     * Node --> Raft 使用 node_tx
    pub async fn poll(&mut self) -> Result<Message, Error> {
        // select!{}
        // incoming_rx: mpsc::Receiver<Message>
        todo!()
    }
}
