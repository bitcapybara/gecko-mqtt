use crate::network::packet::QoS;

pub struct SubAck {
    /// 包 id
    pub packet_id: u16,
    /// 对应于每个订阅时传递的主题，且顺序一致
    pub return_codes: Vec<SubscribeReasonCode>,
}

pub enum SubscribeReasonCode {
    /// 成功，最大服务质量
    Success(QoS),
    /// 失败
    Failure,
}
