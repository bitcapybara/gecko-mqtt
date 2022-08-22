use bytes::{BufMut, BytesMut};

use crate::network::packet::{self, Error};

pub struct PubAck {
    /// 包 id
    pub packet_id: u16,
}

impl PubAck {
    #[inline]
    fn len(&self) -> usize {
        2
    }

    pub fn write(&self, stream: &mut BytesMut) -> Result<(), Error> {
        stream.put_u8(0x40);
        packet::write_remaining_length(stream, self.len())?;
        stream.put_u16(self.packet_id);

        Ok(())
    }
}
