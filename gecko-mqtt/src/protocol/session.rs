use std::collections::HashMap;

use tokio::sync::mpsc::Sender;

use crate::network::{packet, v4};

use super::{subscription::Subscription, Outgoing};

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
    concrete_subscriptions: HashMap<String, packet::QoS>,
    /// 订阅的主题（包含通配符）
    wild_subscriptions: Vec<Subscription>,
    /// 保存发送给客户端但是还没有删除的消息（QoS1, QoS2）(持久化)
    messages: Vec<v4::Publish>,

    /// 发送给客户端的消息
    pub conn_tx: Option<Sender<Outgoing>>,
}

impl Session {
    pub fn new(client_id: &str, clean_session: bool, conn_tx: Sender<Outgoing>) -> Self {
        Self {
            client_id: client_id.into(),
            clean_session,
            concrete_subscriptions: HashMap::new(),
            wild_subscriptions: Vec::new(),
            messages: Vec::new(),
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
}
