use bytes::BytesMut;

use crate::error::Result;

use self::connect::Connect;

mod connect;
mod pingreq;

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PacketType {
    Connect = 1,
    ConnAck,
    Publish,
    PUbAck,
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

enum Version {
    V311,
    V5,
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
    fn read_from(_stream: &mut BytesMut) -> Result<Self> {
        todo!()
    }

    fn packet_type(&self) -> PacketType {
        todo!()
    }
}

pub(crate) enum Packet {
    Connect(Connect),
    ConnAck,
    Publish,
    PUbAck,
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
    fn read_from(stream: &mut BytesMut) -> Result<Self> {
        let fixed_header: FixedHeader = FixedHeader::read_from(stream)?;

        let packet_type = fixed_header.packet_type();

        // 没有负载的 packet 类型
        if fixed_header.remaining_len == 0 {
            return match packet_type {
                PacketType::PingReq => Ok(Packet::PingReq),
                _ => todo!(),
            };
        }

        let packet = match packet_type {
            PacketType::Connect => Packet::Connect(Connect::read_from(stream)?),
            _ => todo!(),
        };

        Ok(packet)
    }
}
