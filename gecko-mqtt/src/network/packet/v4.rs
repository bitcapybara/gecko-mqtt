//! 3.1.1 协议版本报文

use std::slice::Iter;

use bytes::{Buf, BytesMut};

pub use connack::*;
pub use connect::*;
pub use pingresp::*;
pub use puback::*;
pub use pubcomp::*;
pub use publish::*;
pub use pubrec::*;
pub use pubrel::*;
pub use suback::*;
pub use subscribe::*;
pub use unsuback::*;
pub use unsubscribe::*;

pub mod connack;
pub mod connect;
pub mod pingresp;
pub mod puback;
pub mod pubcomp;
pub mod publish;
pub mod pubrec;
pub mod pubrel;
pub mod suback;
pub mod subscribe;
pub mod unsuback;
pub mod unsubscribe;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Invalid packet type: {0}")]
    InvalidPacketType(u8),

    #[error("Invalid protocol")]
    InvalidProtocol,
    #[error("Invalid protocol level: {0}")]
    InvalidProtocolLevel(u8),
    #[error("Incorrect packet format")]
    IncorrectPacketFormat,

    #[error("Payload required")]
    PayloadRequired,

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
    fn packet_type(&self) -> Result<PacketType, super::Error> {
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
            n => Err(Error::InvalidPacketType(n))?,
        }
    }

    /// 整个完整报文的字节长度
    #[inline]
    fn packet_len(&self) -> usize {
        self.fixed_header_len + self.remaining_len
    }
}

impl FixedHeader {
    fn read_from(mut stream: Iter<u8>) -> Result<Self, super::Error> {
        let stream_len = stream.len();
        if stream_len < 2 {
            return Err(super::Error::InsufficientBytes(2 - stream_len));
        }
        // 第一个字节
        let byte1 = stream.next().unwrap();
        // 剩余字节长度
        let mut remaining_len: usize = 0;
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
                return Err(super::Error::MalformedPacket);
            }
        }

        if !done {
            return Err(super::Error::InsufficientBytes(1))?;
        }

        Ok(Self {
            byte1: *byte1,
            fixed_header_len: header_len + 1,
            remaining_len,
        })
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
    Disconnect,
}

impl Packet {
    pub(crate) fn read(stream: &mut BytesMut) -> Result<Self, super::Error> {
        let stream_len = stream.len();
        let fixed_header: FixedHeader = FixedHeader::read_from(stream.iter())?;

        let packet_len = fixed_header.packet_len();
        if stream_len < packet_len {
            return Err(super::Error::InsufficientBytes(packet_len - stream_len))?;
        }

        // 根据固定头给出的长度信息，取出整个报文字节（包含报文头）
        // split_to 方法会更新 stream
        let packet = stream.split_to(fixed_header.packet_len());

        // 报文类型
        let packet_type = fixed_header.packet_type()?;
        // 没有负载的 packet 类型，获取到报文头后，可以直接返回
        if fixed_header.remaining_len == 0 {
            return match packet_type {
                PacketType::PingReq => Ok(Packet::PingReq),
                PacketType::PingResp => Ok(Packet::PingResp),
                PacketType::Disconnect => Ok(Packet::Disconnect),
                _ => Err(Error::PayloadRequired)?,
            };
        }

        // 完整的报文
        let mut stream = packet.freeze();
        // 去掉固定头的报文
        let variable_header_index = fixed_header.fixed_header_len;
        stream.advance(variable_header_index);

        let packet = match packet_type {
            PacketType::Connect => Packet::Connect(Connect::read(stream)?),
            PacketType::Subscribe => Packet::Subscribe(Subscribe::read(stream)?),
            PacketType::Publish => Packet::Publish(Publish::read(fixed_header, stream)?),
            PacketType::PubAck => Packet::PubAck(PubAck::read(fixed_header, stream)?),
            PacketType::PubComp => Packet::PubComp(PubComp::read(fixed_header, stream)?),
            PacketType::PubRec => Packet::PubRec(PubRec::read(fixed_header, stream)?),
            PacketType::PubRel => Packet::PubRel(PubRel::read(fixed_header, stream)?),
            PacketType::Unsubscribe => {
                Packet::Unsubscribe(Unsubscribe::read(fixed_header, stream)?)
            }
            _ => return Err(Error::UnexpectedPacketType)?,
        };

        Ok(packet)
    }

    pub(crate) fn write(&self, stream: &mut BytesMut) -> Result<(), super::Error> {
        match self {
            Packet::ConnAck(ack) => ack.write(stream),
            Packet::PingResp => PingResp.write(stream),
            Packet::SubAck(ack) => ack.write(stream),
            Packet::Publish(publish) => publish.write(stream),
            Packet::PubAck(puback) => puback.write(stream),
            Packet::PubComp(pubcomp) => pubcomp.write(stream),
            Packet::PubRec(pubrec) => pubrec.write(stream),
            Packet::PubRel(pubrel) => pubrel.write(stream),
            Packet::UnsubAck(unsuback) => unsuback.write(stream),
            _ => Err(Error::UnexpectedPacketType)?,
        }
    }

    #[inline]
    pub(crate) fn packet_type(&self) -> PacketType {
        match self {
            Packet::Connect(_) => PacketType::Connect,
            Packet::ConnAck(_) => PacketType::ConnAck,
            Packet::Publish(_) => PacketType::Publish,
            Packet::PubAck(_) => PacketType::PubAck,
            Packet::PubRec(_) => PacketType::PubRec,
            Packet::PubRel(_) => PacketType::PubRel,
            Packet::PubComp(_) => PacketType::PubComp,
            Packet::Subscribe(_) => PacketType::Subscribe,
            Packet::SubAck(_) => PacketType::SubAck,
            Packet::Unsubscribe(_) => PacketType::Unsubscribe,
            Packet::UnsubAck(_) => PacketType::UnsubAck,
            Packet::PingReq => PacketType::PingReq,
            Packet::PingResp => PacketType::PingResp,
            Packet::Disconnect => PacketType::Disconnect,
        }
    }
}
