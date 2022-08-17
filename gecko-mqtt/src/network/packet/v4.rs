//! 3.1.1 协议版本报文

use std::slice::Iter;

use bytes::{Buf, BytesMut};

pub use connack::*;
pub use connect::*;
pub use disconnect::*;
pub use pingreq::*;
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

use super::Error;

pub mod connack;
pub mod connect;
pub mod disconnect;
pub mod pingreq;
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

struct FixedHeader {
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
                return Err(Error::MalformedPacket);
            }
        }

        if !done {
            return Err(Error::InsufficientBytes(1));
        }

        Ok(Self {
            byte1: *byte1,
            fixed_header_len: header_len,
            remaining_len,
        })
    }
}

#[derive(Debug)]
pub enum Packet {
    Connect(Connect),
    ConnAck(ConnAck),
    Publish,
    PubAck,
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

impl Packet {
    pub(crate) fn read_from(stream: &mut BytesMut) -> Result<Self, Error> {
        let fixed_header: FixedHeader = FixedHeader::read_from(stream.iter())?;

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
                _ => Err(Error::PayloadRequired),
            };
        }

        // 完整的报文
        let mut packet = packet.freeze();
        // 去掉固定头的报文
        let variable_header_index = fixed_header.fixed_header_len;
        packet.advance(variable_header_index);

        let packet = match packet_type {
            PacketType::Connect => Packet::Connect(Connect::read_from(packet)?),
            PacketType::ConnAck => todo!(),
            PacketType::Publish => todo!(),
            PacketType::PubAck => todo!(),
            PacketType::PubRec => todo!(),
            PacketType::PubRel => todo!(),
            PacketType::PubComp => todo!(),
            PacketType::Subscribe => todo!(),
            PacketType::SubAck => todo!(),
            PacketType::Unsubscribe => todo!(),
            PacketType::UnsubAck => todo!(),
            PacketType::PingReq => todo!(),
            PacketType::PingResp => todo!(),
            PacketType::Disconnect => todo!(),
        };

        Ok(packet)
    }

    pub(crate) fn write(&self, stream: &mut BytesMut) -> Result<(), Error> {
        match self {
            Packet::ConnAck(ack) => ack.write(stream),
            Packet::PingResp => todo!(),
            Packet::Disconnect => todo!(),
            _ => todo!(),
        }
    }

    #[inline]
    pub(crate) fn packet_type(&self) -> PacketType {
        match self {
            Packet::Connect(_) => PacketType::Connect,
            Packet::ConnAck(_) => PacketType::ConnAck,
            Packet::Publish => PacketType::Publish,
            Packet::PubAck => PacketType::PubAck,
            Packet::PubRel => PacketType::PubRel,
            Packet::PubComp => PacketType::PubComp,
            Packet::Subscribe => PacketType::Subscribe,
            Packet::SubAck => PacketType::SubAck,
            Packet::Unsubscribe => PacketType::Unsubscribe,
            Packet::UnsubAck => PacketType::UnsubAck,
            Packet::PingReq => PacketType::PingReq,
            Packet::PingResp => PacketType::PingResp,
            Packet::Disconnect => PacketType::Disconnect,
        }
    }
}
