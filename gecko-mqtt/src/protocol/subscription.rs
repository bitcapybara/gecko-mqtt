use crate::network::packet;

/// 订阅信息
/// map[topic-filter]Subscription
#[derive(Debug)]
pub struct Subscription {
    client_id: String,
    maximum_qos: packet::QoS,
}
