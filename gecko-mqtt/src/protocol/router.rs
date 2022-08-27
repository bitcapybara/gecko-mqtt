use std::{
    collections::{HashMap, HashSet, VecDeque},
    sync::Arc,
    time,
};

use tokio::{
    select,
    sync::mpsc::{error::SendError, Receiver, Sender},
};

use crate::{
    config,
    network::{
        packet::QoS,
        topic,
        v4::{
            ConnAck, Connect, ConnectReturnCode, Packet, PubAck, PubComp, PubRec, PubRel, Publish,
            SubAck, Subscribe, SubscribeReasonCode, UnsubAck, Unsubscribe,
        },
    },
    Hook,
};

use super::{
    session::{self, Session},
    subscripton::SubscriptionTree,
    Incoming, Outgoing,
};

const SESSION_DEFAULT_EXPIRE_INTERVAL: u64 = 3600;

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
    session_cfg: config::Session,
    /// 各个客户端连接发送过来需要处理的数据
    router_rx: Receiver<Incoming>,
    /// 管理客户端连接信息，key = client_id
    /// session 清理
    /// 将需要清理的session放到一个队列中，队列顺序即代表需要清理的顺序
    /// 当有新的连接进来时，取出队列头的session进行判断清理直到过期时间不满足清理条件，如此，保持内存中的session不会引起大的内存泄漏
    sessions: HashMap<String, Session>,
    /// 已经失效的 session，等待超时移除 (client_id, push_to_queue_time)
    ineffective_sessions: VecDeque<(String, time::Instant)>,

    /// TODO 加速消息发布查找
    /// 全局的精确订阅信息, key = topic-filter, value = client_id
    concrete_subscriptions: HashMap<String, HashSet<String>>,
    /// 全局的模糊订阅信息, T = client_id
    wild_subscriptions: SubscriptionTree<String>,

    /// 保留消息
    retains: Vec<Publish>,
    /// 钩子函数
    hook: Arc<H>,
}

impl<H: Hook> Router<H> {
    pub(crate) fn new(
        session_cfg: config::Session,
        hook: Arc<H>,
        router_rx: Receiver<Incoming>,
    ) -> Self {
        Self {
            session_cfg,
            router_rx,
            sessions: HashMap::new(),
            ineffective_sessions: VecDeque::new(),
            concrete_subscriptions: HashMap::new(),
            wild_subscriptions: SubscriptionTree::new(),
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
                        Packet::Unsubscribe(unsubscribe) => {
                            self.handle_unsubscribe(&client_id, unsubscribe).await?
                        }
                        Packet::Disconnect => self.handle_client_disconnect(&client_id).await?,
                        _ => return Err(Error::UnexpectedPacket),
                    }
                }
                Ok(())
            }
            Incoming::Disconnect { client_id } => self.handle_conn_disconnect(&client_id).await,
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
        // 拿出当前存储的 session（没来得及清理）
        let session = match self.sessions.remove(&client_id) {
            Some(session) => {
                // 客户端断开了，但是服务端还没察觉到，会发生 conn_tx 还存在这种情况
                if let Some(conn_tx) = &session.conn_tx {
                    let _ = conn_tx.try_send(Outgoing::Disconnect);
                }
                if !clean_session {
                    Some(session)
                } else {
                    None
                }
            }
            None => None,
        };
        // 从待清理队列中移除当前会话
        self.ineffective_sessions.retain(|(c, _)| c != &client_id);
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

        // 清理一波旧的 session
        let now = time::Instant::now();
        while let Some((client_id, ineffected_at)) = self.ineffective_sessions.pop_front() {
            // 没到超时时间，退出
            let session_expire_interval = self
                .session_cfg
                .expire_interval
                .unwrap_or(SESSION_DEFAULT_EXPIRE_INTERVAL);
            if now.duration_since(ineffected_at)
                < time::Duration::from_secs(session_expire_interval)
            {
                self.ineffective_sessions
                    .push_front((client_id.clone(), ineffected_at));
                break;
            }
            // 超时的，删除
            self.sessions.remove(&client_id);
        }
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

        match self.sessions.get_mut(client_id) {
            Some(session) => {
                let mut return_codes = Vec::with_capacity(filters.len());
                for filter in filters {
                    let path = filter.path;
                    // 添加到订阅管理
                    if topic::filter_has_wildcards(&path) {
                        // 查询一下是否已添加过
                        if session.wildcard_subscriptions.contains_key(&path) {
                            continue;
                        }
                        // 添加到全局
                        let token = self.wild_subscriptions.insert(&path, client_id.into());
                        // 添加到session里
                        session.wildcard_subscriptions.insert(path, token);
                    } else {
                        // 查询一下是否已添加过
                        if session.concrete_subscriptions.contains(&path) {
                            continue;
                        }
                        // 添加到全局
                        if let Some(clients) = self.concrete_subscriptions.get_mut(&path) {
                            clients.insert(client_id.into());
                        } else {
                            let new_set = HashSet::from_iter([client_id.into()]);
                            self.concrete_subscriptions.insert(path.clone(), new_set);
                        }
                        // 添加到session里
                        session.concrete_subscriptions.insert(path);
                    }
                    // TODO 添加一些校验，目前 sub 都是 success
                    return_codes.push(SubscribeReasonCode::Success(filter.qos));
                }

                let ack = SubAck {
                    packet_id,
                    return_codes,
                };
                Ok(session.send_packet(Packet::SubAck(ack)).await?)
            }
            None => Err(Error::SessionNotFound),
        }
    }

    async fn handle_unsubscribe(
        &mut self,
        client_id: &str,
        unsubscribe: Unsubscribe,
    ) -> Result<(), Error> {
        let Unsubscribe { packet_id, filters } = unsubscribe;
        match self.sessions.get_mut(client_id) {
            Some(session) => {
                for filter in filters {
                    if topic::filter_has_wildcards(&filter) {
                        if let Some(token) = session.wildcard_subscriptions.remove(&filter) {
                            self.wild_subscriptions.remove(&filter, token);
                        }
                    } else {
                        session.concrete_subscriptions.remove(&filter);
                        self.concrete_subscriptions.remove(&filter);
                    }
                }
                Ok(session
                    .send_packet(Packet::UnsubAck(UnsubAck { packet_id }))
                    .await?)
            }
            None => Err(Error::SessionNotFound),
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
        let Publish { topic, .. } = publish;
        let mut clients = HashSet::new();
        // 精确匹配
        if let Some(client_ids) = self.concrete_subscriptions.get(topic) {
            for client_id in client_ids {
                clients.insert(client_id);
            }
        }
        // 模糊匹配
        clients.extend(self.wild_subscriptions.matches(topic));

        // 发送
        for client_id in clients {
            if let Some(session) = self.sessions.get_mut(client_id) {
                session.publish_message(publish).await?;
            }
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
    /// 不发送 will 消息
    async fn handle_client_disconnect(&mut self, client_id: &str) -> Result<(), Error> {
        if let Some(session) = self.sessions.get_mut(client_id) {
            // 向 conn 返回断开连接确认消息
            if let Some(conn_tx) = session.conn_tx.take() {
                conn_tx.send(Outgoing::Disconnect).await?
            }

            // 放到会话失效列表
            self.ineffective_sessions
                .push_back((session.client_id.clone(), time::Instant::now()));
        }

        Ok(())
    }

    /// 处理客户端的异常退出，发送 will 消息
    ///
    /// 如：
    /// * 协议格式错误
    /// * 网络错误
    async fn handle_conn_disconnect(&mut self, client_id: &str) -> Result<(), Error> {
        if let Some(session) = self.sessions.get_mut(client_id) {
            self.ineffective_sessions
                .push_back((session.client_id.clone(), time::Instant::now()));
        }

        // TODO 发送 will 消息

        Ok(())
    }
}
