use std::collections::HashSet;

use tokio::sync::mpsc::Receiver;

use super::Incoming;

/// 处理 mqtt 协议层运行时相关逻辑
/// 接收消息，处理，发送到对应的设备/节点
pub(crate) struct Router {
    /// 各个客户端连接发送过来需要处理的数据
    incoming_rx: Receiver<Incoming>,
    /// 管理客户端连接信息，
    conns: HashSet<usize>,
}
