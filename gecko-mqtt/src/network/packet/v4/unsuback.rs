use bytes::{BufMut, BytesMut};

use crate::network::packet::Error;

#[derive(Debug)]
pub struct UnsubAck {
    /// 包 id
    pub packet_id: u16,
}

impl UnsubAck {
    pub fn write(&self, stream: &mut BytesMut) -> Result<(), Error> {
        stream.put_slice(&[0xB0, 0x02]);
        stream.put_u16(self.packet_id);
        Ok(())
    }
}
