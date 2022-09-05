use std::slice::Iter;

use bytes::{Buf, BufMut, Bytes, BytesMut};

pub mod v4;
pub mod v5;

const PAYLOAD_MAX_LENGTH: usize = 268_435_455;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("At least {0} more bytes required")]
    InsufficientBytes(usize),
    #[error("Malformed UTF-8 string")]
    MalformedString,
    #[error("Malformed packet")]
    MalformedPacket,
    #[error("Invalid QoS: {0}")]
    InvalidQoS(u8),
    #[error("Payload too large")]
    PayloadTooLarge,
    #[error("Invalid protocol")]
    InvalidProtocol,
    #[error("Invalid protocol level: {0}")]
    InvalidProtocolLevel(u8),
    #[error("Invalid packet type: {0}")]
    InvalidPacketType(u8),
    #[error("Invalid v4 packet: {0}")]
    V4(#[from] v4::Error),
    #[error("Invalid v5 packet: {0}")]
    V5(#[from] v5::Error),
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

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PacketType {
    Connect = 1,
    ConnAck,
    Publish,
    PubAck,
    PubRec,
    PubRel,
    PubComp,
    Subscribe,
    SubAck,
    Unsubscribe,
    UnsubAck,
    PingReq,
    PingResp,
    Disconnect,
}

#[derive(Debug)]
pub struct FixedHeader {
    /// 固定头的第一个字节，包含报文类型和flags
    byte1: u8,
    // 固定头的大小
    fixed_header_len: usize,
    // 剩余长度大小
    remaining_len: usize,
}

impl FixedHeader {
    #[inline]
    fn packet_type(&self) -> Result<PacketType, Error> {
        let num = self.byte1 >> 4;
        match num {
            1 => Ok(PacketType::Connect),
            2 => Ok(PacketType::ConnAck),
            3 => Ok(PacketType::Publish),
            4 => Ok(PacketType::PubAck),
            5 => Ok(PacketType::PubRec),
            6 => Ok(PacketType::PubRel),
            7 => Ok(PacketType::PubComp),
            8 => Ok(PacketType::Subscribe),
            9 => Ok(PacketType::SubAck),
            10 => Ok(PacketType::Unsubscribe),
            11 => Ok(PacketType::UnsubAck),
            12 => Ok(PacketType::PingReq),
            13 => Ok(PacketType::PingResp),
            14 => Ok(PacketType::Disconnect),
            n => Err(Error::InvalidPacketType(n)),
        }
    }

    /// 整个完整报文的字节长度
    #[inline]
    fn packet_len(&self) -> usize {
        self.fixed_header_len + self.remaining_len
    }
}

impl FixedHeader {
    fn read_from(mut stream: Iter<u8>) -> Result<Self, Error> {
        let stream_len = stream.len();
        if stream_len < 2 {
            return Err(Error::InsufficientBytes(2 - stream_len));
        }
        // 第一个字节
        let byte1 = stream.next().unwrap();
        let (remaining_len, header_len) = length(stream)?;

        Ok(Self {
            byte1: *byte1,
            fixed_header_len: header_len + 1,
            remaining_len,
        })
    }
}

#[derive(Debug)]
pub enum Packet {
    V4(Box<v4::Packet>),
    V5(Box<v5::Packet>),
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

fn read_u32(stream: &mut Bytes) -> Result<u32, Error> {
    if stream.len() < 4 {
        return Err(Error::MalformedPacket);
    }

    Ok(stream.get_u32())
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

pub fn length(stream: Iter<u8>) -> Result<(usize, usize), Error> {
    // 剩余字节长度
    let mut remaining_len = 0;
    // 固定头长度
    let mut header_len = 0;
    let mut done = false;
    let mut shift = 0;

    for byte in stream {
        // 固定头长度 + 1
        header_len += 1;
        // 剩余长度字节
        let byte = *byte as usize;
        // 字节的后七位 * 128 + 上一个字节
        remaining_len += (byte & 0x7F) << shift;

        // 是否还有后续 remining_len 字节
        done = (byte & 0x80) == 0;
        if done {
            break;
        }

        shift += 7;

        // 剩余长度字节最多四个字节（0，7，14，21）
        if shift > 21 {
            return Err(Error::MalformedPacket);
        }
    }

    if !done {
        return Err(Error::InsufficientBytes(1));
    }

    Ok((header_len, remaining_len))
}
