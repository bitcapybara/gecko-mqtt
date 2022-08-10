use bytes::BytesMut;

use super::Error;

#[derive(Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq, Hash)]
pub(crate) struct Subscribe {
    /// 订阅的 topic
    topic: String,
    /// 订阅的服务质量
    qos: u8,
}

impl Subscribe {
    pub(crate) fn read_from(_stream: &mut BytesMut) -> Result<Self, Error> {
        todo!()
    }
}
