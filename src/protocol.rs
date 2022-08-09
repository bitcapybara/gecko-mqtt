//! 协议层
//! 处理协议相关的逻辑，依赖于底层的网络层进行网络读写

use tokio::sync::mpsc::{Receiver, Sender};

use crate::{error::Result, network};

pub use session::SessionState;

mod router;
mod session;

pub(crate) type ConnectionId = usize;

/// 发送给 router 的消息
enum Incoming {}

/// router 发送给客户端的回复
enum Outcoming {}

/// 一个客户端或对等节点连接的**事件循环**
pub(crate) struct ConnectionEventLoop {
    /// 当前客户端或对等节点连接对应的底层网络连接
    conn: network::Connection,

    /// 给 router 发送消息的管道
    router_tx: Sender<Incoming>,
    /// 从协议层接收到的消息
    conn_rx: Receiver<Outcoming>,
}

impl ConnectionEventLoop {
    /// 开启事件循环
    /// * 从 conn socket 网络层获取 packet 数据，发送给 router
    /// * 接收 router 的回复，写入 conn socket 网络层
    pub(crate) async fn start(&mut self) -> Result<()> {
        // loop {
        //     let packets = self.conn.read_packets().await?;

        // }
        todo!()
    }
}
