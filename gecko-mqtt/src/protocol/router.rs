use std::{collections::HashMap, sync::Arc};

use log::warn;
use tokio::{
    select,
    sync::mpsc::{error::SendError, Receiver, Sender},
};

use crate::{
    network::v4::{ConnAck, Connect, ConnectReturnCode},
    Hook,
};

use super::{session::Session, Incoming, Outgoing};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Failed to send outgoing message: {0}")]
    SendOutgoing(#[from] SendError<Outgoing>),
}

/// 处理 mqtt 协议层运行时相关逻辑
/// 接收消息，处理，发送到对应的设备/节点
pub(crate) struct Router<H: Hook> {
    /// 各个客户端连接发送过来需要处理的数据
    router_rx: Receiver<Incoming>,
    /// 管理客户端连接信息，key = client_id
    sessions: HashMap<String, Session>,
    /// 钩子函数
    hook: Arc<H>,
}

impl<H: Hook> Router<H> {
    pub(crate) fn new(hook: Arc<H>, router_rx: Receiver<Incoming>) -> Self {
        Self {
            router_rx,
            sessions: HashMap::new(),
            hook,
        }
    }

    /// 开始 router 逻辑处理循环
    pub(crate) async fn start(mut self) -> Result<(), Error> {
        loop {
            select! {
                // 接收客户端连接发来的消息
                recv = self.router_rx.recv() => {
                    match recv {
                        Some(incoming) => self.handle_incoming(incoming).await?,
                        None => todo!(),
                    }
                }
            }
        }
    }

    /// 分发处理
    async fn handle_incoming(&mut self, incoming: Incoming) -> Result<(), Error> {
        match incoming {
            Incoming::Connect {
                connect: packet,
                conn_tx,
            } => self.handle_connect(packet, conn_tx).await,
            Incoming::Data(_) => todo!(),
        }
    }

    /// 处理客户端连接
    async fn handle_connect(
        &mut self,
        connect: Connect,
        conn_tx: Sender<Outgoing>,
    ) -> Result<(), Error> {
        let client_id = connect.client_id;
        let clean_session = connect.clean_session;
        // 拿出当前存储的 session
        let session = if let Some(session) = self.sessions.remove(&client_id) {
            if let Some(conn_tx) = &session.conn_tx {
                if let Err(e) = conn_tx.try_send(Outgoing::Disconnect) {
                    warn!(
                        "Failed to send disconnect packet to token-over session: {0}",
                        e
                    )
                }
            }
            if clean_session {
                Some(session)
            } else {
                None
            }
        } else {
            None
        };
        let session_present = session.is_some();

        // TODO 清理 session 中还积压的消息

        // 发送 ack 消息
        let ack = ConnAck {
            session_present,
            code: ConnectReturnCode::Success,
        };
        conn_tx.send(Outgoing::ConnAck(ack)).await?;

        let new_session = match session {
            Some(s) => s.into_new(clean_session, conn_tx),
            None => Session::new(&client_id, clean_session, conn_tx),
        };

        self.sessions.insert(client_id, new_session);
        Ok(())
    }
}
