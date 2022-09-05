//! 3.1.1 协议版本报文

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

use super::PacketType;

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
        let fixed_header = super::FixedHeader::read_from(stream.iter())?;

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
