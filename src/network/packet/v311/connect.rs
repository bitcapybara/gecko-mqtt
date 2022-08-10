use bytes::{Bytes, BytesMut};

use crate::network::packet::{Error, Protocol, QoS};

pub struct Connect {
    /// 协议版本
    pub protocol: Protocol,
    /// keepalive
    pub keep_alive: u16,
    /// 客户端id
    pub client_id: String,
    /// 是否开启新会话
    pub clean_session: bool,
    /// 遗嘱消息
    pub last_will: Option<LastWill>,
    /// 登录凭证
    pub login: Option<Login>,
}

impl Connect {
    pub(crate) fn read_from(_stream: &mut BytesMut) -> Result<Self, Error> {
        todo!()
    }
}

/// 遗嘱设置
pub struct LastWill {
    /// 遗嘱发送的目标主题
    pub topic: String,
    // 遗嘱消息
    pub message: Bytes,
    /// 服务质量
    pub qos: QoS,
    /// 消息保留
    pub retain: bool,
}

/// 登录凭证
pub struct Login {
    /// 用户名
    pub username: String,
    /// 密码
    pub password: String,
}
