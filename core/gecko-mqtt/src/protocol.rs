//! 协议层
//! 处理协议相关的逻辑，依赖于底层的网络层进行网络读写

use tokio::sync::mpsc::Sender;

use crate::network::v4::{ConnAck, Connect, Packet};

pub(crate) use router::Router;
pub use session::SessionState;

mod router;
mod session;

pub(crate) type ConnectionId = usize;

/// 发送给 router 的消息
#[derive(Debug)]
pub enum Incoming {
    Connect {
        packet: Connect,
        conn_tx: Sender<Outgoing>,
    },
    Data(Vec<Packet>),
}

/// router 发送给客户端的回复
#[derive(Debug)]
pub enum Outgoing {
    ConnAck { id: ConnectionId, packet: ConnAck },
    Data(Packet),
}
