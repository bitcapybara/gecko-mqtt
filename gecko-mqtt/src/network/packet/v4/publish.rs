use bytes::{BufMut, Bytes, BytesMut};

use crate::network::{
    packet::{self, Error, QoS},
    topic,
};

use super::FixedHeader;

#[derive(Debug, Clone)]
pub struct Publish {
    /// 客户端是否之前发送过此消息（是否重新投递）
    pub dup: bool,
    /// 服务质量
    pub qos: QoS,
    /// 消息保留
    pub retain: bool,
    /// 主题
    pub topic: String,
    /// 包 id
    pub packet_id: u16,
    /// 消息负载
    pub payload: Bytes,
}

impl Publish {
    fn len(&self) -> usize {
        let mut len = 2 + self.topic.len();
        if self.qos != QoS::AtMostOnce && self.packet_id != 0 {
            len += 2;
        }
        len += self.payload.len();

        len
    }

    pub fn read(fixed_header: FixedHeader, mut stream: Bytes) -> Result<Self, Error> {
        let byte1 = fixed_header.byte1;
        let qos = ((byte1 & 0b0110) >> 1).try_into()?;
        let dup = (byte1 & 0b1000) != 0;
        let retain = (byte1 & 0b0001) != 0;

        let topic = packet::read_string(&mut stream)?;
        if !topic::valid_publish_topic(&topic) {
            return Err(Error::InvalidPublishTopic);
        }
        let packet_id = match qos {
            QoS::AtMostOnce => 0,
            QoS::AtLeastOnce | QoS::ExactlyOnce => {
                let pkid = packet::read_u16(&mut stream)?;
                if pkid == 0 {
                    return Err(Error::MissPacketId);
                }
                pkid
            }
        };

        Ok(Self {
            dup,
            qos,
            retain,
            topic,
            packet_id,
            payload: stream,
        })
    }

    pub fn write(&self, stream: &mut BytesMut) -> Result<(), Error> {
        let len = self.len();

        let dup = self.dup as u8;
        let qos = self.qos as u8;
        let retain = self.retain as u8;
        stream.put_u8(0b0011_0000 | retain | qos << 1 | dup << 3);

        packet::write_remaining_length(stream, len)?;
        packet::write_string(stream, &self.topic);

        if self.qos != QoS::AtMostOnce {
            let pkid = self.packet_id;
            if pkid == 0 {
                return Err(Error::MissPacketId);
            }

            stream.put_u16(pkid);
        }

        stream.extend_from_slice(&self.payload);

        Ok(())
    }
}
