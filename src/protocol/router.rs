use std::collections::HashSet;

use tokio::sync::mpsc::Receiver;

use super::Incoming;

/// 维护 mqtt 运行时的所有全局信息
/// 处理 mqtt 协议层相关逻辑
pub(crate) struct Router {
    /// 各个客户端连接发送过来需要处理的数据
    incoming_rx: Receiver<Incoming>,
    /// 管理客户端连接信息，
    conns: HashSet<usize>,
}
