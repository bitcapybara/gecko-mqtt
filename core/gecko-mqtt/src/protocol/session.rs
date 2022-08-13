use std::{collections::HashSet, time::Duration};

use tokio::sync::mpsc::Sender;

use crate::{cluster::Storage, network::v4};

use super::{ConnectionId, Outgoing};

/// session 会话
/// 每个客户端必定对应一个会话
/// clean_session = false 需要持久化
/// clean_session = true 内存保存即可
pub struct SessionState {
    /// 客户端id（客户端生成）,immutable
    client_id: String,
    /// clean session（持久化）,immutable
    clean_session: bool,
    /// 订阅的主题 topic,qos（持久化）
    subscribes: HashSet<v4::Subscribe>,
    /// 保存发送给客户端但是还没有删除的消息（QoS1, QoS2）(持久化)
    messages: Vec<v4::Publish>,
    /// 设备断开连接的时间戳，毫秒
    /// 如果存在表明设备断开了连接，开始计时直到会话过期删除
    /// TODO 考虑 broker 崩溃重启的情况
    disconnect_at: Option<u128>,
    /// 过期时长
    expire: Duration,
}

/// 代表服务端的一次会话
/// 会话的生命周期不能小于一次客户端连接
/// 处理协议层客户端逻辑，如 QoS1, QoS2 的消息保存等
/// 协议层会话和网络层连接通过 ConnectionEventLoop 进行通信
pub struct Session {
    /// 客户端连接 id（服务端分配）
    id: ConnectionId,
    /// 会话状态
    state: SessionState,

    /// 发送给客户端的消息
    conn_tx: Sender<Outgoing>,
    /// 持久化存储
    persist: Storage,
}

impl Session {
    /// 传入的 topic 是否与客户端订阅匹配
    fn topic_match(_topic: String) -> bool {
        todo!()
    }

    /// 客户端与服务端的连接断开，通知 session
    fn delete_client() {
        todo!()
    }
}
