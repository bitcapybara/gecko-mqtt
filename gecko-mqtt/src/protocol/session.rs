use std::collections::HashSet;

use tokio::sync::mpsc::Sender;

use crate::network::v4;

use super::Outgoing;

/// session 会话
/// 每个客户端必定对应一个会话
/// clean_session = false 需要持久化
/// clean_session = true 内存保存即可
pub struct SessionState {
    /// 订阅的主题 topic,qos（持久化）
    subscribes: HashSet<v4::Subscribe>,
    /// 保存发送给客户端但是还没有删除的消息（QoS1, QoS2）(持久化)
    messages: Vec<v4::Publish>,
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
    /// 会话状态
    state: SessionState,
    /// 过期配置

    /// 发送给客户端的消息
    pub conn_tx: Option<Sender<Outgoing>>,
}

impl Session {
    pub fn new(client_id: &str, clean_session: bool, conn_tx: Sender<Outgoing>) -> Self {
        Self {
            client_id: client_id.into(),
            clean_session,
            state: SessionState {
                subscribes: HashSet::new(),
                messages: Vec::new(),
            },
            conn_tx: Some(conn_tx),
        }
    }

    pub fn into_new(self, clean_session: bool, conn_tx: Sender<Outgoing>) -> Self {
        Self {
            client_id: self.client_id,
            clean_session,
            state: self.state,
            conn_tx: Some(conn_tx),
        }
    }
}
