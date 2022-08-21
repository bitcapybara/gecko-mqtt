use bytes::{BufMut, BytesMut};

use crate::network::packet::{self, Error, QoS};

#[derive(Debug)]
pub struct SubAck {
    /// 包 id
    pub packet_id: u16,
    /// 对应于每个订阅时传递的主题，且顺序一致
    pub return_codes: Vec<SubscribeReasonCode>,
}

impl SubAck {
    pub fn write(&self, stream: &mut BytesMut) -> Result<(), Error> {
        stream.put_u8(0x90);
        let remaining_len = self.len();
        packet::write_remaining_length(stream, remaining_len)?;

        stream.put_u16(self.packet_id);
        let p = self
            .return_codes
            .iter()
            .map(|&code| match code {
                SubscribeReasonCode::Success(qos) => qos as u8,
                SubscribeReasonCode::Failure => 0x80,
            })
            .collect::<Vec<u8>>();
        stream.extend_from_slice(&p);
        Ok(())
    }

    pub fn len(&self) -> usize {
        2 + self.return_codes.len()
    }
}

#[derive(Debug, Clone, Copy)]
pub enum SubscribeReasonCode {
    /// 成功，最大服务质量
    Success(QoS),
    /// 失败
    Failure,
}
