use bytes::{BufMut, Bytes, BytesMut};

use crate::network::packet::{self, Error};

use super::FixedHeader;

#[derive(Debug)]
pub struct PubRel {
    /// 包 id
    pub packet_id: u16,
}

impl PubRel {
    #[inline]
    fn len(&self) -> usize {
        2
    }

    pub fn read(fixed_header: FixedHeader, mut stream: Bytes) -> Result<Self, Error> {
        let packet_id = packet::read_u16(&mut stream)?;
        if fixed_header.remaining_len != 2 {
            return Err(Error::MalformedPacket);
        }

        Ok(PubRel { packet_id })
    }

    pub fn write(&mut self, stream: &mut BytesMut) -> Result<(), Error> {
        stream.put_u8(0x62);
        packet::write_remaining_length(stream, self.len())?;
        stream.put_u16(self.packet_id);
        Ok(())
    }
}
