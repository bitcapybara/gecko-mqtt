//! 来自对等节点的连接

use tokio::sync::mpsc::Receiver;

use crate::cluster::Dispatcher;

use crate::error::Result;
use crate::network::v4::Packet;

/// 计划使用 grpc Unary Rpc
///
/// 接收到请求后再通过 channel 传过来
pub(crate) struct PeerConnection {
    /// 接收到对等节点发来的数据
    stream: Receiver<()>,
    /// 数据发送给对等节点
    dispatcher: Dispatcher,
}

impl PeerConnection {
    /// 从已读取的缓冲区中获取 packet 存入列表
    pub(crate) async fn read_packets(&mut self) -> Result<Vec<Packet>> {
        todo!()
    }
}
