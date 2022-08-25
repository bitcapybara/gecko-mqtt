use std::collections::HashMap;

pub struct SubscriptionTree<T> {
    root: Option<SubscriptionNode<T>>,
    /// 插入的每一个数据，分配一个唯一的 token 号，方便查询和删除
    token: u64
}

/// 订阅树的节点
/// 每个客户端订阅的 filter 不固定，可能会挂在任何一个节点上
pub struct SubscriptionNode<T> {
    // 当前节点包含的片段
    key: LevelKey,
    /// 当前节点包含的数据（客户端信息）
    /// key = token, value = data
    data: HashMap<u64, T>,
    /// 子节点 key = level-key
    children: HashMap<String, SubscriptionNode<T>>,
}

pub enum LevelKey {
    Concrete(String),
    SingleLevelWildcard,
    MultiLevelWildcard,
}

impl<T> SubscriptionTree<T> {
    pub fn new() -> Self {
        Self { root: None, token: 0 }
    }

    /// 插入一个订阅记录
    pub fn insert(_filter: &str, _data:T) -> u64 {
        todo!()
    }

    /// 查找发布消息的主题匹配的记录
    pub fn search(_topic: &str) -> Option<Vec<&T>> {
        None
    }

    /// 删除订阅记录
    pub fn remove(_filter: &str, _token: u64) {
        todo!()
    }
}
