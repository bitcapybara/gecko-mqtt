use std::{collections::HashMap, sync::Arc};

use tokio::sync::mpsc::Receiver;

use crate::Hook;

use super::{session::Session, ConnectionId, Incoming};

#[derive(Debug, thiserror::Error)]
pub enum Error {}

/// 处理 mqtt 协议层运行时相关逻辑
/// 接收消息，处理，发送到对应的设备/节点
pub(crate) struct Router<H: Hook> {
    /// 各个客户端连接发送过来需要处理的数据
    router_rx: Receiver<Incoming>,
    /// 管理客户端连接信息，
    conns: HashMap<ConnectionId, Session>,
    /// 钩子函数
    hook: Option<Arc<H>>,
}

impl<H: Hook> Router<H> {
    pub(crate) fn new(hook: Option<Arc<H>>, router_rx: Receiver<Incoming>) -> Self {
        Self {
            router_rx,
            conns: HashMap::new(),
            hook,
        }
    }

    /// 开始 router 洛基家处理循环
    pub(crate) async fn start(&self) -> Result<(), Error> {
        todo!()
    }
}
