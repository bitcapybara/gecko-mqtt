use std::{collections::HashMap, sync::Arc};

use log::warn;
use tokio::{
    select,
    sync::mpsc::{error::SendError, Receiver, Sender},
};

use crate::{
    network::{
        packet::QoS,
        v4::{
            ConnAck, Connect, ConnectReturnCode, Packet, PubAck, PubComp, PubRec, PubRel, Publish,
            SubAck, Subscribe, SubscribeReasonCode,
        },
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
    /// 保留消息
    retains: Vec<Publish>,
    /// 钩子函数
    hook: Arc<H>,
}

impl<H: Hook> Router<H> {
    pub(crate) fn new(hook: Arc<H>, router_rx: Receiver<Incoming>) -> Self {
        Self {
            router_rx,
            sessions: HashMap::new(),
            retains: Vec::new(),
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
                        Packet::PubAck(puback) => self.handle_publish_ack(&client_id, puback),
                        Packet::PubRel(pubrel) => {
                            self.handle_publish_release(&client_id, pubrel).await?
                        }
                        Packet::PubRec(pubrec) => {
                            self.handle_publish_receive(&client_id, pubrec).await?
                        }
                        Packet::PubComp(pubcomp) => {
                            self.handle_publish_complete(&client_id, pubcomp)
                        }
                        Packet::Disconnect => self.handle_disconnect(&client_id, false).await?,
                        _ => return Err(Error::UnexpectedPacket),
                    }
                }
                Ok(())
            }
            Incoming::Disconnect { client_id } => self.handle_disconnect(&client_id, true).await,
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
    ///
    /// QoS0：发送端 和 接受端 均不保存数据
    /// QoS1：发送端 保存数据，接受端 不保存
    /// QoS2：发送端 和 接受端 均保存数据
    async fn handle_publish(&mut self, client_id: &str, publish: Publish) -> Result<(), Error> {
        let Publish {
            retain,
            packet_id,
            qos,
            ..
        } = publish;

        // 保留消息，router 保存一份
        if retain {
            self.retains.push(publish.clone());
        }

        // 回复 publisher
        match qos {
            QoS::AtMostOnce => {
                // 给订阅端发送消息
                self.publish_message(&publish).await?
            }
            QoS::AtLeastOnce => {
                if let Some(session) = self.sessions.get_mut(client_id) {
                    // broker 是接收端，不需要保存消息，直接发送 puback
                    session
                        .send_packet(Packet::PubAck(PubAck { packet_id }))
                        .await?;
                    // 给订阅端发送消息
                    self.publish_message(&publish).await?
                }
            }
            QoS::ExactlyOnce => {
                if let Some(session) = self.sessions.get_mut(client_id) {
                    // 保存起来，下次接收到 pubrel 消息时删除
                    session.insert_received(packet_id);
                    // 发送 pubrec
                    session
                        .send_packet(Packet::PubRec(PubRec { packet_id }))
                        .await?;
                }
            }
        }

        Ok(())
    }

    /// 给所有符合条件的客户端发送消息
    async fn publish_message(&mut self, publish: &Publish) -> Result<(), Error> {
        for session in self.sessions.values_mut() {
            session.publish_message(publish).await?;
        }
        Ok(())
    }

    /// 处理 puback
    fn handle_publish_ack(&mut self, client_id: &str, puback: PubAck) {
        if let Some(session) = self.sessions.get_mut(client_id) {
            session.remove_published(puback.packet_id);
        }
    }

    /// 处理 pubrel
    async fn handle_publish_release(
        &mut self,
        client_id: &str,
        pubrel: PubRel,
    ) -> Result<(), Error> {
        if let Some(session) = self.sessions.get_mut(client_id) {
            session.publish_release(pubrel).await?;
        }

        Ok(())
    }

    /// 处理 pubrec
    async fn handle_publish_receive(
        &mut self,
        client_id: &str,
        pubrec: PubRec,
    ) -> Result<(), Error> {
        if let Some(session) = self.sessions.get_mut(client_id) {
            session.publish_receive(pubrec).await?;
        }

        Ok(())
    }

    /// 处理 pubcomp
    fn handle_publish_complete(&mut self, client_id: &str, pubcomp: PubComp) {
        if let Some(session) = self.sessions.get_mut(client_id) {
            session.publish_complete(pubcomp);
        }
    }

    /// 处理客户端断开连接事件
    /// exec_will true 表示客户端已异常退出，session 不需要再发送消息给 conn_tx
    /// exec_will false 表示客户端主动断开连接，session 需要发送回 conn_tx 一个 Disconnect 消息使 conn 正常退出
    /// session 处理完后，即可 drop 掉 conn_tx
    /// 
    /// TODO session 清理
    /// 将需要清理的session放到一个队列中，队列顺序即代表需要清理的顺序
    /// 当有新的连接进来时，取出队列头的session进行判断清理直到过期时间不满足清理条件，如此，保持内存中的session不会引起大的内存泄漏
    async fn handle_disconnect(&mut self, _client_id: &str, _exec_will: bool) -> Result<(), Error> {
        todo!()
    }
}
