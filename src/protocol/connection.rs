use tokio::sync::mpsc::Sender;

use super::Outcoming;

/// 代表了一个客户端连接
/// 处理一些客户端逻辑，如
pub(crate) struct Connection {
    /// 客户端连接 id
    id: usize,
    /// 发送给客户端的消息
    conn_tx: Sender<Outcoming>,
}
