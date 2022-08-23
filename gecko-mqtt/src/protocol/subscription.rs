use crate::network::packet::QoS;

/// 订阅信息
/// map[topic-filter]Subscription
#[derive(Debug)]
pub struct Subscription {
    /// 客户端 id
    client_id: String,
    /// 订阅的 topic filter
    filter: String,
    /// Node Id 用于分布式
    /// 服务质量
    qos: QoS,
}
