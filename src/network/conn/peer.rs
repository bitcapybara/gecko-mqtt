//! 来自对等节点的连接

use tokio::sync::mpsc::Receiver;

use crate::cluster::Dispatcher;

/// 计划使用 grpc Unary Rpc
///
/// 接收到请求后再通过 channel 传过来
pub(crate) struct PeerConnection {
    /// 接收到对等节点发来的数据
    stream: Receiver<()>,
    /// 数据发送给对等节点
    dispatcher: Dispatcher,
}
