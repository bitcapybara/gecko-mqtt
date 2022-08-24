use bytes::{Buf, Bytes};

use crate::network::{
    packet::{self, read_u8, Error, QoS},
    topic,
};

#[derive(Debug)]
pub struct Subscribe {
    pub packet_id: u16,
    pub filters: Vec<SubscribeFilter>,
}

impl Subscribe {
    pub fn read(mut stream: Bytes) -> Result<Self, Error> {
        let packet_id = packet::read_u16(&mut stream)?;

        let mut filters = Vec::new();
        while stream.has_remaining() {
            let filter = packet::read_string(&mut stream)?;
            if !topic::valid_subscribe_filter(&filter) {
                return Err(Error::InvalidSubscribeFilter);
            }
            let options = read_u8(&mut stream)?;
            let qos = options & 0b0000_0011;

            filters.push(SubscribeFilter {
                path: filter,
                qos: qos.try_into()?,
            })
        }

        Ok(Self { packet_id, filters })
    }
}

#[derive(Debug)]
pub struct SubscribeFilter {
    pub path: String,
    pub qos: QoS,
}
