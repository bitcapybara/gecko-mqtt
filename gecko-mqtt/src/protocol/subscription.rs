use crate::network::packet;

/// 订阅信息
/// map[topic-filter]Subscription
#[derive(Debug)]
pub struct Subscription {
    /// 订阅的 topic filter
    topic_filter: String,
    /// Node Id 用于分布式
    /// 服务质量
    qos: packet::QoS,
}
