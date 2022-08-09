use bytes::{Buf, BytesMut};

use crate::error::Result;

pub(crate) use connect::Connect;
pub(crate) use publish::Publish;
pub(crate) use subscribe::Subscribe;

mod connect;
mod pingreq;
pub(crate) mod publish;
pub(crate) mod subscribe;

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

/// 服务质量
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, serde::Serialize, serde::Deserialize)]
#[allow(clippy::enum_variant_names)]
pub(crate) enum QoS {
    AtMostOnce = 1,
    AtLeastOnce,
    ExactlyOnce,
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
    fn read_from(stream: &mut BytesMut) -> Result<Self> {
        let stream_len = stream.len();
        if stream_len < 2 {
            // return err 字节不足
            todo!()
        }
        // 第一个字节
        let byte1 = stream.get_u8();

        let mut remaining_len: usize = 0;
        let mut header_len = 0;
        let mut done = false;
        let mut shift = 0;
        while stream.has_remaining() {
            // 固定头长度 + 1
            header_len += 1;
            // 剩余长度字节
            let byte = stream.get_u8() as usize;
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
                // err
                todo!()
            }
        }

        if !done {
            // err
            todo!()
        }

        Ok(Self {
            byte1,
            fixed_header_len: header_len,
            remaining_len,
        })
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
