use std::collections::HashMap;

use packet::v4::{Packet, Publish};
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
    client_id: String,
    /// clean session（持久化）,immutable
    clean_session: bool,
    /// 过期配置

    /// 订阅的主题（精确匹配）
    concrete_subscriptions: HashMap<String, QoS>,
    /// 订阅的主题（包含通配符）
    wild_subscriptions: HashMap<String, QoS>,
    /// 保存发送给客户端但是还没有删除的消息（QoS1, QoS2）(持久化)
    /// TODO 过期处理？
    messages: HashMap<u16, Publish>,

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
            messages: HashMap::new(),
            conn_tx: Some(conn_tx),
        }
    }

    pub fn into_new(self, clean_session: bool, conn_tx: Sender<Outgoing>) -> Self {
        Self {
            client_id: self.client_id,
            clean_session,
            concrete_subscriptions: self.concrete_subscriptions,
            wild_subscriptions: self.wild_subscriptions,
            messages: self.messages,
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

    /// 给客户端发送消息
    pub async fn send_packet(&self, packet: Packet) -> Result<(), Error> {
        if let Some(ref sender) = self.conn_tx {
            Ok(sender.send(Outgoing::Data(packet)).await?)
        } else {
            Err(Error::SessionConnTxNotFound)
        }
    }

    /// 匹配 publish 的 topic
    ///
    /// * qos0: publish
    /// * qos1: store, publish, puback
    /// * qos2: store, pubrec
    pub async fn handle_publish(&mut self, publish: &Publish) -> Result<Option<QoS>, Error> {
        let Publish {
            qos,
            topic,
            packet_id,
            ..
        } = publish;

        // 匹配
        let mut downgrade_qos = None;
        if let Some(sub_qos) = self.concrete_subscriptions.get(topic) {
            downgrade_qos.replace(sub_qos.downgrade(qos));
        }

        if downgrade_qos.is_none() {
            for filter in self.wild_subscriptions.iter() {
                if topic::matches(topic, filter.0) {
                    downgrade_qos.replace(filter.1.downgrade(qos));
                    break;
                }
            }
        }

        if let Some(qos) = downgrade_qos {
            match qos {
                QoS::AtMostOnce => {
                    self.send_packet(Packet::Publish(publish.clone())).await?;
                }
                QoS::AtLeastOnce => {
                    self.messages.insert(packet_id.to_owned(), publish.clone());
                    self.send_packet(Packet::Publish(publish.clone())).await?;
                }
                QoS::ExactlyOnce => {
                    self.messages.insert(packet_id.to_owned(), publish.clone());
                }
            }
        }

        Ok(downgrade_qos.cloned())
    }
}
