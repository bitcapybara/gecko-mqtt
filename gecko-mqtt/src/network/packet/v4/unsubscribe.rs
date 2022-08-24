use bytes::Bytes;

use crate::network::packet::{self, Error};

use super::FixedHeader;

#[derive(Debug)]
pub struct Unsubscribe {
    /// 包 id
    pub packet_id: u16,
    /// 取消订阅的主题
    pub filters: Vec<String>,
}

impl Unsubscribe {
    pub fn read(fixed_header: FixedHeader, mut stream: Bytes) -> Result<Self, Error> {
        let packet_id = packet::read_u16(&mut stream)?;
        let mut payload_len = fixed_header.remaining_len - 2;
        let mut filters = Vec::with_capacity(1);

        while payload_len > 0 {
            let filter = packet::read_string(&mut stream)?;
            payload_len -= filter.len() + 2;
            filters.push(filter);
        }

        Ok(Self { packet_id, filters })
    }
}
