use bytes::{BufMut, Bytes, BytesMut};

use crate::network::packet::{self, Error, FixedHeader};

use super::PacketProperties;

#[derive(Debug)]
pub struct PubRel {
    pub packet_id: u16,
    pub reason: PubRelReason,
    pub properties: Option<PacketProperties>,
}

impl PubRel {
    fn len(&self) -> usize {
        if self.reason == PubRelReason::Success && self.properties.is_none() {
            return 2;
        }

        let mut len = 2 + 1;
        match &self.properties {
            Some(properties) => {
                let properties_len = properties.len();
                let properties_len_len = super::len_len(properties_len);
                len += properties_len_len + properties_len;
            }
            None => len += 1,
        }

        len
    }

    pub fn read(fixed_header: FixedHeader, mut stream: Bytes) -> Result<Self, Error> {
        let packet_id = packet::read_u16(&mut stream)?;
        if fixed_header.remaining_len == 2 {
            return Ok(Self {
                packet_id,
                reason: PubRelReason::Success,
                properties: None,
            });
        }

        let reason = packet::read_u8(&mut stream)?;
        if fixed_header.remaining_len < 4 {
            return Ok(Self {
                packet_id,
                reason: PubRelReason::Success,
                properties: None,
            });
        }

        Ok(Self {
            packet_id,
            reason: reason.try_into()?,
            properties: PacketProperties::read(&mut stream)?,
        })
    }

    pub fn write(&self, stream: &mut BytesMut) -> Result<(), Error> {
        stream.put_u8(0x62);
        packet::write_remaining_length(stream, self.len())?;
        stream.put_u16(self.packet_id);

        if self.reason == PubRelReason::Success && self.properties.is_none() {
            return Ok(());
        }

        stream.put_u8(self.reason as u8);
        match &self.properties {
            Some(properties) => properties.write(stream)?,
            None => {
                packet::write_remaining_length(stream, 0)?;
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum PubRelReason {
    Success = 0,
    PacketIdentifierNotFound = 146,
}

impl TryFrom<u8> for PubRelReason {
    type Error = super::Error;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        let code = match value {
            0 => PubRelReason::Success,
            146 => PubRelReason::PacketIdentifierNotFound,
            num => return Err(super::Error::InvalidReasonCode(num)),
        };

        Ok(code)
    }
}
