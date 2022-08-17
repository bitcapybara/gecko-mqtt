use bytes::{Bytes, BytesMut};

use crate::network::packet::{Error, QoS};

#[derive(Debug)]
pub struct Publish {
    /// 客户端是否之前发送过此消息（是否重新投递）
    pub dup: bool,
    /// 服务质量
    pub qos: QoS,
    /// 消息保留
    pub retain: bool,
    /// 主题
    pub topic: String,
    /// 包 id
    pub packet_id: u16,
    /// 消息负载
    pub payload: Bytes,
}

impl Publish {
    pub(crate) fn read_from(_stream: &mut BytesMut) -> Result<Self, Error> {
        todo!()
    }
}
