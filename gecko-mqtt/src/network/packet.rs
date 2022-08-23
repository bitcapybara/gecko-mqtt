use bytes::{Buf, BufMut, Bytes, BytesMut};

pub mod v4;
pub mod v5;

const PAYLOAD_MAX_LENGTH: usize = 268_435_455;

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
    #[error("Payload required")]
    PayloadRequired,
    #[error("Payload too large")]
    PayloadTooLarge,
    #[error("Payload size incorrect")]
    PayloadSizeIncorrect,
    #[error("Unexpected packet type")]
    UnexpectedPacketType,
    #[error("Miss packet id")]
    MissPacketId,
    #[error("Invalid publish topic")]
    InvalidPublishTopic,
    #[error("Invalid subscribe filter")]
    InvalidSubscribeFilter,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Protocol {
    /// v3.1.1
    V4,
    /// v5
    V5,
}

/// 服务质量
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[allow(clippy::enum_variant_names)]
pub enum QoS {
    AtMostOnce = 0,
    AtLeastOnce,
    ExactlyOnce,
}

impl QoS {
    pub fn downgrade<'a>(&'a self, qos: &'a QoS) -> &QoS {
        match self {
            QoS::AtMostOnce => self,
            QoS::AtLeastOnce => match qos {
                QoS::AtMostOnce => qos,
                _ => self,
            },
            QoS::ExactlyOnce => qos,
        }
    }
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

/// 读取多个字节
fn read_bytes(stream: &mut Bytes) -> Result<Bytes, Error> {
    // 后续可取出的字节的长度
    let len = read_u16(stream)? as usize;

    if len > stream.len() {
        return Err(Error::MalformedPacket);
    }

    Ok(stream.split_to(len))
}

fn read_string(stream: &mut Bytes) -> Result<String, Error> {
    let s = read_bytes(stream)?;
    match String::from_utf8(s.to_vec()) {
        Ok(v) => Ok(v),
        Err(_) => Err(Error::MalformedString),
    }
}

fn read_u16(stream: &mut Bytes) -> Result<u16, Error> {
    if stream.len() < 2 {
        return Err(Error::MalformedPacket);
    }

    Ok(stream.get_u16())
}

fn read_u8(stream: &mut Bytes) -> Result<u8, Error> {
    if stream.is_empty() {
        return Err(Error::MalformedPacket);
    }
    Ok(stream.get_u8())
}

fn write_remaining_length(stream: &mut BytesMut, len: usize) -> Result<usize, Error> {
    if len > PAYLOAD_MAX_LENGTH {
        return Err(Error::PayloadTooLarge);
    }

    let mut done = false;
    let mut x = len;
    let mut count = 0;

    while !done {
        let mut byte = (x % 128) as u8;
        x /= 128;
        if x > 0 {
            byte |= 128;
        }

        stream.put_u8(byte);
        count += 1;
        done = x == 0;
    }

    Ok(count)
}

fn write_bytes(stream: &mut BytesMut, bytes: &[u8]) {
    stream.put_u16(bytes.len() as u16);
    stream.extend_from_slice(bytes);
}

fn write_string(stream: &mut BytesMut, string: &str) {
    write_bytes(stream, string.as_bytes())
}
