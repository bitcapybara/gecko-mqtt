#[derive(Debug)]
pub struct Unsubscribe {
    /// 包 id
    pub packet_id: u16,
    /// 取消订阅的主题
    pub filters: Vec<String>,
}
