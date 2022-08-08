#[derive(Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq, Hash)]
pub(crate) struct Subscribe {
    /// 订阅的 topic
    topic: String,
    /// 订阅的服务质量
    qos: u8,
}
