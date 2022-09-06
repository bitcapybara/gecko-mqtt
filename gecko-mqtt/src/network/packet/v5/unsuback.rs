use bytes::{BufMut, BytesMut};

use crate::network::packet::{self, Error};

use super::PacketProperties;

#[derive(Debug)]
pub struct UnsubAck {
    pub packet_id: u16,
    pub reasons: Vec<UnsubAckReason>,
    pub properties: Option<PacketProperties>,
}

impl UnsubAck {
    fn len(&self) -> usize {
        let mut len = 2 + self.reasons.len();

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

    pub fn write(&self, stream: &mut BytesMut) -> Result<(), Error> {
        stream.put_u8(0x80);
        packet::write_remaining_length(stream, self.len())?;

        stream.put_u16(self.packet_id);

        match &self.properties {
            Some(properties) => properties.write(stream)?,
            None => {
                packet::write_remaining_length(stream, 0)?;
            }
        }

        let codes = self
            .reasons
            .iter()
            .map(|code| *code as u8)
            .collect::<Vec<u8>>();
        stream.extend_from_slice(&codes);

        Ok(())
    }
}

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum UnsubAckReason {
    Success = 0x00,
    NoSubscriptionExisted = 0x11,
    UnspecifiedError = 0x80,
    ImplementationSpecificError = 0x83,
    NotAuthorized = 0x87,
    TopicFilterInvalid = 0x8F,
    PacketIdentifierInUse = 0x91,
}

impl TryFrom<u8> for UnsubAckReason {
    type Error = super::Error;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        let code = match value {
            0x00 => UnsubAckReason::Success,
            0x11 => UnsubAckReason::NoSubscriptionExisted,
            0x80 => UnsubAckReason::UnspecifiedError,
            0x83 => UnsubAckReason::ImplementationSpecificError,
            0x87 => UnsubAckReason::NotAuthorized,
            0x8F => UnsubAckReason::TopicFilterInvalid,
            0x91 => UnsubAckReason::PacketIdentifierInUse,
            num => return Err(super::Error::InvalidReasonCode(num)),
        };

        Ok(code)
    }
}
