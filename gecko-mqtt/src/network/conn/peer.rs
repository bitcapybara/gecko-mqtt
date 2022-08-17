//! 来自对等节点的连接

use tokio::sync::mpsc::{Receiver, Sender};

use crate::cluster::Dispatcher;
use crate::network::v4::Packet;
use crate::protocol::Incoming;

use super::Error;

/// 计划使用 grpc Unary Rpc
///
/// 接收到请求后再通过 channel 传过来
pub(crate) struct PeerConnection {
    /// 接收到对等节点发来的数据
    peer_rx: Receiver<()>,
    /// 数据发送给对等节点
    dispatcher: Dispatcher,
}

impl PeerConnection {
    pub(crate) fn new(peer_rx: Receiver<()>) -> Self {
        Self {
            peer_rx,
            dispatcher: Dispatcher::new(),
        }
    }

    pub(crate) fn start(self, _router_tx: Sender<Incoming>) -> Result<(), Error> {
        todo!()
    }

    /// 从已读取的缓冲区中获取 packet 存入列表
    pub(crate) async fn read_packets(&mut self) -> Result<Vec<Packet>, Error> {
        todo!()
    }
}
