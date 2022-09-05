use bytes::{BufMut, Bytes, BytesMut};

use crate::network::packet::{self, Error};

use packet::FixedHeader;

#[derive(Debug)]
pub struct PubRec {
    /// 包 id
    pub packet_id: u16,
}

impl PubRec {
    fn len(&self) -> usize {
        2
    }

    pub fn read(fixed_header: FixedHeader, mut stream: Bytes) -> Result<Self, Error> {
        let packet_id = packet::read_u16(&mut stream)?;
        if fixed_header.remaining_len != 2 {
            return Err(Error::MalformedPacket);
        }

        Ok(Self { packet_id })
    }

    pub fn write(&self, stream: &mut BytesMut) -> Result<(), Error> {
        stream.put_u8(0x50);
        packet::write_remaining_length(stream, self.len())?;
        stream.put_u16(self.packet_id);
        Ok(())
    }
}
