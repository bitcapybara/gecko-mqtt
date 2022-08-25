pub struct SubscriptionTree<T> {
    root: Option<SubscriptionNode<T>>,
}

pub struct SubscriptionNode<T> {
    // 当前节点包含的片段
    level: Level,
    // 如果当前节点是叶子节点时，包含的数据
    data: Option<T>,
}

pub enum Level {
    Concrete(String),
    SingleLevelWildcard,
    MultiLevelWildcard,
}

impl<T> SubscriptionTree<T> {
    pub fn new() -> Self {
        Self { root: None }
    }
}
