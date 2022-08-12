use bytes::{Buf, Bytes, BytesMut};

pub mod v4;
pub mod v5;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Invalid packet type: {0}")]
    InvalidPacketType(u8),
    #[error("Malformed packet")]
    MalformedPacket,
    #[error("At least {0} more bytes required")]
    InsufficientBytes(usize),
    #[error("Malformed UTF-8 string")]
    MalformedString,
    #[error("Invalid protocol")]
    InvalidProtocol,
    #[error("Invalid protocol level: {0}")]
    InvalidProtocolLevel(u8),
    #[error("Incorrect packet format")]
    IncorrectPacketFormat,
    #[error("Invalid QoS: {0}")]
    InvalidQoS(u8),
}

pub enum Protocol {
    V4,
    V5,
}

/// 服务质量
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd)]
#[allow(clippy::enum_variant_names)]
pub enum QoS {
    AtMostOnce = 1,
    AtLeastOnce,
    ExactlyOnce,
}

impl TryFrom<u8> for QoS {
    type Error = Error;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(QoS::AtMostOnce),
            1 => Ok(QoS::AtLeastOnce),
            2 => Ok(QoS::ExactlyOnce),
            qos => Err(Error::InvalidQoS(qos)),
        }
    }
}

fn read_bytes(stream: &mut BytesMut) -> Result<Bytes, Error> {
    let len = read_u16(stream)? as usize;

    if len > stream.len() {
        return Err(Error::MalformedPacket);
    }

    let bs = stream.get(0..len).unwrap().to_vec();
    stream.advance(len);

    Ok(Bytes::from(bs))
}

fn read_string(stream: &mut BytesMut) -> Result<String, Error> {
    let s = read_bytes(stream)?;
    match String::from_utf8(s.to_vec()) {
        Ok(v) => Ok(v),
        Err(_) => Err(Error::MalformedString),
    }
}

fn read_u16(stream: &mut BytesMut) -> Result<u16, Error> {
    if stream.is_empty() {
        return Err(Error::MalformedPacket);
    }
    if stream.len() < 2 {
        return Err(Error::InsufficientBytes(2 - stream.len()));
    }

    Ok(stream.get_u16())
}

fn read_u8(stream: &mut BytesMut) -> Result<u8, Error> {
    if stream.is_empty() {
        return Err(Error::MalformedPacket);
    }
    Ok(stream.get_u8())
}
