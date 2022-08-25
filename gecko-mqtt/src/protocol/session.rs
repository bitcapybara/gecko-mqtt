use std::collections::{HashMap, HashSet};

use packet::v4::{Packet, PubComp, PubRec, PubRel, Publish};
use tokio::sync::mpsc::{error::SendError, Sender};

use crate::network::{
    packet::{self, QoS},
    topic,
};

use super::Outgoing;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Failed to send outgoing message: {0}")]
    SendOutgoing(#[from] SendError<Outgoing>),
    #[error("Session conn tx not found")]
    SessionConnTxNotFound,
}

/// 代表服务端的一次会话
/// 会话的生命周期不能小于一次客户端连接
/// 处理协议层客户端逻辑，如 QoS1, QoS2 的消息保存等
/// 协议层会话和网络层连接通过 ConnectionEventLoop 进行通信
pub struct Session {
    /// 客户端 id
    pub client_id: String,
    /// clean session（持久化）,immutable
    clean_session: bool,

    /// 订阅的主题（精确匹配）
    concrete_subscriptions: HashMap<String, QoS>,
    /// 订阅的主题（包含通配符）
    /// TODO 使用 订阅树？
    wild_subscriptions: HashMap<String, QoS>,

    /// 保存发送给客户端但是还没有删除的消息（QoS1, QoS2）(持久化)
    /// 接收到 puback/pubcomp 后删除
    messages_publish: HashMap<u16, Publish>,
    /// 在收到 qos2 publish 的消息时保存，在收到 qos2 pubrelease 的消息后删除
    messages_receive: HashSet<u16>,
    /// 在收到 qos2 pubrec 的消息时保存，在收到 qos2 pubcomp 的消息后删除
    messages_release: HashSet<u16>,

    /// 发送给客户端的消息
    pub conn_tx: Option<Sender<Outgoing>>,
}

impl Session {
    pub fn new(client_id: &str, clean_session: bool, conn_tx: Sender<Outgoing>) -> Self {
        Self {
            client_id: client_id.into(),
            clean_session,
            concrete_subscriptions: HashMap::new(),
            wild_subscriptions: HashMap::new(),
            messages_publish: HashMap::new(),
            messages_receive: HashSet::new(),
            messages_release: HashSet::new(),
            conn_tx: Some(conn_tx),
        }
    }

    pub fn into_new(self, clean_session: bool, conn_tx: Sender<Outgoing>) -> Self {
        Self {
            client_id: self.client_id,
            clean_session,
            concrete_subscriptions: self.concrete_subscriptions,
            wild_subscriptions: self.wild_subscriptions,
            messages_publish: self.messages_publish,
            messages_receive: self.messages_receive,
            messages_release: self.messages_release,
            conn_tx: Some(conn_tx),
        }
    }

    /// 添加订阅topic，如果相同则覆盖
    /// 即，同一个会话中，不可以有多个一样的 topic filter
    pub fn insert_filter(&mut self, filter: (&str, QoS)) {
        let topic_filter = filter.0.into();
        let qos = filter.1;
        if topic::topic_has_wildcards(filter.0) {
            self.wild_subscriptions.insert(topic_filter, qos);
        } else {
            self.concrete_subscriptions.insert(topic_filter, qos);
        }
    }

    pub fn remove_filters(&mut self, filters: &[String]) {
        for filter in filters {
            self.concrete_subscriptions.remove(filter);
            self.wild_subscriptions.remove(filter);
        }
    }

    /// 给客户端发送消息
    pub async fn send_packet(&self, packet: Packet) -> Result<(), Error> {
        if let Some(ref sender) = self.conn_tx {
            Ok(sender.send(Outgoing::Data(packet)).await?)
        } else {
            Err(Error::SessionConnTxNotFound)
        }
    }

    ///
    pub fn insert_received(&mut self, packet_id: u16) {
        self.messages_receive.insert(packet_id);
    }

    pub fn remove_received(&mut self, packet_id: u16) {
        self.messages_receive.insert(packet_id);
    }

    pub fn remove_published(&mut self, packet_id: u16) {
        self.messages_publish.remove(&packet_id);
    }

    /// 匹配 publish 的 topic
    ///
    /// * qos0: publish
    /// * qos1: store, publish, puback
    /// * qos2: store, pubrec
    pub async fn publish_message(&mut self, publish: &Publish) -> Result<(), Error> {
        let Publish {
            qos,
            topic,
            packet_id,
            ..
        } = publish;

        // 查询是否匹配
        let mut matched = self.concrete_subscriptions.contains_key(topic);
        if !matched {
            for filter in self.wild_subscriptions.iter() {
                if topic::matches(topic, filter.0) {
                    matched = true;
                    break;
                }
            }
        }

        // 根据订阅的qos处理
        if matched {
            match qos {
                QoS::AtMostOnce => {
                    // 发送给订阅的客户端
                    self.send_packet(Packet::Publish(publish.clone())).await?;
                }
                QoS::AtLeastOnce => {
                    // 保存起来，等待接收到 puback/pubcomp 后删除
                    self.messages_publish
                        .insert(packet_id.to_owned(), publish.clone());
                    // 发送给订阅的客户端
                    self.send_packet(Packet::Publish(publish.clone())).await?;
                }
                QoS::ExactlyOnce => {
                    // 保存数据，收到 pubrel 后再发送
                    self.messages_publish
                        .insert(packet_id.to_owned(), publish.clone());
                }
            }
        }

        Ok(())
    }

    pub async fn publish_release(&mut self, pubrel: PubRel) -> Result<(), Error> {
        if self.messages_receive.remove(&pubrel.packet_id) {
            self.send_packet(Packet::PubComp(PubComp {
                packet_id: pubrel.packet_id,
            }))
            .await?;
        }
        Ok(())
    }

    pub async fn publish_receive(&mut self, pubrec: PubRec) -> Result<(), Error> {
        if self.messages_publish.contains_key(&pubrec.packet_id) {
            self.messages_release.insert(pubrec.packet_id);
            self.send_packet(Packet::PubRel(PubRel {
                packet_id: pubrec.packet_id,
            }))
            .await?;
        }

        Ok(())
    }

    pub fn publish_complete(&mut self, pubcomp: PubComp) {
        if self.messages_release.remove(&pubcomp.packet_id) {
            self.messages_publish.remove(&pubcomp.packet_id);
        }
    }
}
