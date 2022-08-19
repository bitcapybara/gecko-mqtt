use crate::network::packet;

/// 订阅信息
/// map[topic-filter]Subscription
#[derive(Debug)]
pub struct Subscription {
    /// 客户端 id
    client_id: String,
    /// Node Id 用于分布式
    /// 服务质量
    qos: packet::QoS,
}
