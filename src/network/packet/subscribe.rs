use bytes::BytesMut;

use super::{Error, QoS};

pub struct Subscribe {
    pub pkid: u16,
    pub filters: Vec<TopicFilter>,
}

impl Subscribe {
    fn read_from(_stream: &mut BytesMut) -> Result<Self, Error> {
        todo!()
    }
}

pub struct TopicFilter {
    pub filter: String,
    pub qos: QoS,
}
