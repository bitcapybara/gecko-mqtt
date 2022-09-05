//! v5 协议版本报文

use bytes::{Buf, BufMut, Bytes, BytesMut};

use self::{
    connack::ConnAck, connect::Connect, disconnect::Disconnect, puback::PubAck, pubcomp::PubComp,
    publish::Publish, pubrec::PubRec, pubrel::PubRel, suback::SubAck, subscribe::Subscribe,
    unsuback::UnsubAck, unsubscribe::Unsubscribe,
};

mod connack;
mod connect;
mod disconnect;
mod pingresp;
mod puback;
mod pubcomp;
mod publish;
mod pubrec;
mod pubrel;
mod suback;
mod subscribe;
mod unsuback;
mod unsubscribe;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Invalid property type: {0}")]
    UnexpectedPropertyType(u8),
    #[error("Invalid reason code: {0}")]
    InvalidReasonCode(u8),
}

#[repr(u8)]
#[derive(Debug)]
enum PropertyType {
    PayloadFormatIndicator = 1,
    MessageExpiryInterval = 2,
    ContentType = 3,
    ResponseTopic = 8,
    CorrelationData = 9,
    SubscriptionIdentifier = 11,
    SessionExpiryInterval = 17,
    AssignedClientIdentifier = 18,
    ServerKeepAlive = 19,
    AuthenticationMethod = 21,
    AuthenticationData = 22,
    RequestProblemInformation = 23,
    WillDelayInterval = 24,
    RequestResponseInformation = 25,
    ResponseInformation = 26,
    ServerReference = 28,
    ReasonString = 31,
    ReceiveMaximum = 33,
    TopicAliasMaximum = 34,
    TopicAlias = 35,
    MaximumQos = 36,
    RetainAvailable = 37,
    UserProperty = 38,
    MaximumPacketSize = 39,
    WildcardSubscriptionAvailable = 40,
    SubscriptionIdentifierAvailable = 41,
    SharedSubscriptionAvailable = 42,
}

impl TryFrom<u8> for PropertyType {
    type Error = super::Error;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        let property = match value {
            1 => PropertyType::PayloadFormatIndicator,
            2 => PropertyType::MessageExpiryInterval,
            3 => PropertyType::ContentType,
            8 => PropertyType::ResponseTopic,
            9 => PropertyType::CorrelationData,
            11 => PropertyType::SubscriptionIdentifier,
            17 => PropertyType::SessionExpiryInterval,
            18 => PropertyType::AssignedClientIdentifier,
            19 => PropertyType::ServerKeepAlive,
            21 => PropertyType::AuthenticationMethod,
            22 => PropertyType::AuthenticationData,
            23 => PropertyType::RequestProblemInformation,
            24 => PropertyType::WillDelayInterval,
            25 => PropertyType::RequestResponseInformation,
            26 => PropertyType::ResponseInformation,
            28 => PropertyType::ServerReference,
            31 => PropertyType::ReasonString,
            33 => PropertyType::ReceiveMaximum,
            34 => PropertyType::TopicAliasMaximum,
            35 => PropertyType::TopicAlias,
            36 => PropertyType::MaximumQos,
            37 => PropertyType::RetainAvailable,
            38 => PropertyType::UserProperty,
            39 => PropertyType::MaximumPacketSize,
            40 => PropertyType::WildcardSubscriptionAvailable,
            41 => PropertyType::SubscriptionIdentifierAvailable,
            42 => PropertyType::SharedSubscriptionAvailable,
            num => return Err(Error::UnexpectedPropertyType(num))?,
        };

        Ok(property)
    }
}

#[derive(Debug)]
pub enum Packet {
    Connect(Connect),
    ConnAck(ConnAck),
    Publish(Publish),
    PubAck(PubAck),
    PubRec(PubRec),
    PubRel(PubRel),
    PubComp(PubComp),
    Subscribe(Subscribe),
    SubAck(SubAck),
    Unsubscribe(Unsubscribe),
    UnsubAck(UnsubAck),
    PingReq,
    PingResp,
    Disconnect(Disconnect),
}

#[derive(Debug)]
pub struct PacketProperties {
    pub reason_string: Option<String>,
    pub user_properties: Vec<(String, String)>,
}

impl PacketProperties {
    pub fn len(&self) -> usize {
        let mut len = 0;

        if let Some(reason) = &self.reason_string {
            len += 1 + 2 + reason.len();
        }

        for (key, value) in &self.user_properties {
            len += 1 + 2 + key.len() + 2 + value.len();
        }

        len
    }

    pub fn read(stream: &mut Bytes) -> Result<Option<Self>, super::Error> {
        let mut reason_string = None;
        let mut user_properties = Vec::new();

        let (properties_len_len, properties_len) = super::length(stream.iter())?;
        stream.advance(properties_len_len);
        if properties_len == 0 {
            return Ok(None);
        }

        let mut cursor = 0;
        while cursor < properties_len {
            let prop = super::read_u8(stream)?;
            cursor += 1;

            match prop.try_into()? {
                PropertyType::ReasonString => {
                    let reason = super::read_string(stream)?;
                    cursor += 2 + reason.len();
                    reason_string = Some(reason);
                }
                PropertyType::UserProperty => {
                    let key = super::read_string(stream)?;
                    let value = super::read_string(stream)?;
                    cursor += 2 + key.len() + 2 + value.len();
                    user_properties.push((key, value));
                }
                _ => return Err(Error::UnexpectedPropertyType(prop))?,
            }
        }

        Ok(Some(Self {
            reason_string,
            user_properties,
        }))
    }

    pub fn write(&self, stream: &mut BytesMut) -> Result<(), super::Error> {
        super::write_remaining_length(stream, self.len())?;

        if let Some(reason) = &self.reason_string {
            stream.put_u8(PropertyType::ReasonString as u8);
            super::write_string(stream, reason);
        }

        for (key, value) in &self.user_properties {
            stream.put_u8(PropertyType::UserProperty as u8);
            super::write_string(stream, key);
            super::write_string(stream, value);
        }

        Ok(())
    }
}

fn len_len(len: usize) -> usize {
    if len >= 2_097_152 {
        4
    } else if len >= 16_384 {
        3
    } else if len >= 128 {
        2
    } else {
        1
    }
}
