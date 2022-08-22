use std::{collections::HashMap, sync::Arc};

use log::warn;
use tokio::{
    select,
    sync::mpsc::{error::SendError, Receiver, Sender},
};

use crate::{
    network::v4::{
        ConnAck, Connect, ConnectReturnCode, Packet, Publish, SubAck, Subscribe,
        SubscribeReasonCode,
    },
    Hook,
};

use super::{
    session::{self, Session},
    Incoming, Outgoing,
};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Failed to send outgoing message: {0}")]
    SendOutgoing(#[from] SendError<Outgoing>),
    #[error("Unexpected packet")]
    UnexpectedPacket,
    #[error("Session not found")]
    SessionNotFound,
    #[error("session error: {0}")]
    Session(#[from] session::Error),
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
            Incoming::Connect { connect, conn_tx } => self.handle_connect(connect, conn_tx).await,
            Incoming::Data { client_id, packets } => {
                for packet in packets.into_iter() {
                    match packet {
                        Packet::Subscribe(subscribe) => {
                            self.handle_subscribe(&client_id, subscribe).await?
                        }
                        Packet::Publish(publish) => {
                            self.handle_publish(&client_id, publish).await?
                        }
                        _ => return Err(Error::UnexpectedPacket),
                    }
                }
                Ok(())
            }
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
                    warn!("Failed to send disconnect packet to old session: {0}", e)
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

    /// 处理订阅请求
    /// TODO 给订阅的客户端发送所有匹配的保留消息
    async fn handle_subscribe(
        &mut self,
        client_id: &str,
        subscribe: Subscribe,
    ) -> Result<(), Error> {
        let Subscribe { packet_id, filters } = subscribe;

        if let Some(session) = self.sessions.get_mut(client_id) {
            let mut return_codes = Vec::with_capacity(filters.len());
            for filter in filters {
                // 添加到 session
                session.insert_filter((&filter.path, filter.qos));
                // TODO 添加一些校验，目前 sub 都是 success
                return_codes.push(SubscribeReasonCode::Success(filter.qos));
            }

            let ack = SubAck {
                packet_id,
                return_codes,
            };

            Ok(session.send_packet(Packet::SubAck(ack)).await?)
        } else {
            Err(Error::SessionNotFound)
        }
    }

    /// 处理 publish 请求
    async fn handle_publish(&mut self, _client_id: &str, _publish: Publish) -> Result<(), Error> {
        todo!()
    }
}
