use bytes::{BufMut, BytesMut};

use crate::network::packet::{self, Error};

use super::PacketProperties;

#[derive(Debug)]
pub struct SubAck {
    pub packet_id: u16,
    pub return_codes: Vec<SubscribeReasonCode>,
    pub properties: Option<PacketProperties>,
}

impl SubAck {
    fn len(&self) -> usize {
        let mut len = 2 + self.return_codes.len();

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
        stream.put_u8(0x90);
        packet::write_remaining_length(stream, self.len())?;

        stream.put_u16(self.packet_id);

        match &self.properties {
            Some(properties) => properties.write(stream)?,
            None => {
                packet::write_remaining_length(stream, 0)?;
            }
        }

        let codes = self
            .return_codes
            .iter()
            .map(|code| *code as u8)
            .collect::<Vec<u8>>();
        stream.extend_from_slice(&codes);

        Ok(())
    }
}

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum SubscribeReasonCode {
    QoS0 = 0,
    QoS1 = 1,
    QoS2 = 2,
    Unspecified = 128,
    ImplementationSpecific = 131,
    NotAuthorized = 135,
    TopicFilterInvalid = 143,
    PkidInUse = 145,
    QuotaExceeded = 151,
    SharedSubscriptionsNotSupported = 158,
    SubscriptionIdNotSupported = 161,
    WildcardSubscriptionsNotSupported = 162,
}

impl TryFrom<u8> for SubscribeReasonCode {
    type Error = super::Error;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        let v = match value {
            0 => SubscribeReasonCode::QoS0,
            1 => SubscribeReasonCode::QoS1,
            2 => SubscribeReasonCode::QoS2,
            128 => SubscribeReasonCode::Unspecified,
            131 => SubscribeReasonCode::ImplementationSpecific,
            135 => SubscribeReasonCode::NotAuthorized,
            143 => SubscribeReasonCode::TopicFilterInvalid,
            145 => SubscribeReasonCode::PkidInUse,
            151 => SubscribeReasonCode::QuotaExceeded,
            158 => SubscribeReasonCode::SharedSubscriptionsNotSupported,
            161 => SubscribeReasonCode::SubscriptionIdNotSupported,
            162 => SubscribeReasonCode::WildcardSubscriptionsNotSupported,
            v => return Err(super::Error::InvalidReasonCode(v)),
        };

        Ok(v)
    }
}
