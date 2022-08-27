use std::{collections::HashMap, fmt::Debug};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Parse topic level error")]
    ParseFilterLevel,
}

#[derive(Debug, serde::Serialize)]
pub struct SubscriptionTree<T: Debug> {
    /// 订阅树的根节点，是个空节点
    root: SubscriptionNode<T>,
    /// 插入的每一个数据，分配一个唯一的 token 号，方便查询和删除
    token: u64,
}

impl<T: Debug> SubscriptionTree<T> {
    pub fn new() -> Self {
        Self {
            root: SubscriptionNode::empty(),
            token: 0,
        }
    }

    /// 插入一个订阅记录
    pub fn insert(&mut self, filter: &str, data: T) -> u64 {
        let token = self.token;
        self.root.insert(filter, token, data);
        self.token += 1;

        token
    }

    /// 查找发布消息的主题匹配的记录
    pub fn matches(&mut self, topic: &str) -> Vec<&T> {
        self.root.matches(topic.split('/'))
    }

    /// 删除订阅记录
    pub fn remove(&mut self, filter: &str, token: u64) {
        self.root.remove(filter.split('/'), token)
    }
}

/// 订阅树的节点
/// 每个客户端订阅的 filter 不固定，可能会挂在任何一个节点上
#[derive(Debug, serde::Serialize)]
pub struct SubscriptionNode<T: Debug> {
    // 当前节点包含的片段，根节点为 None
    key: Option<LevelKey>,
    /// 当前节点包含的数据（客户端信息）
    /// key = token, value = data
    data: HashMap<u64, T>,
    /// 子节点 key = level-key
    children: HashMap<String, SubscriptionNode<T>>,
}

impl<T: Debug> SubscriptionNode<T> {
    fn empty() -> Self {
        Self {
            key: None,
            data: HashMap::with_capacity(0),
            children: HashMap::new(),
        }
    }

    fn new(key: LevelKey) -> Self {
        Self {
            key: Some(key),
            data: HashMap::new(),
            children: HashMap::new(),
        }
    }

    /// 将订阅加到当前节点的子树中
    fn insert(&mut self, filter: &str, token: u64, data: T) {
        let mut current_node = self;

        for path in filter.split('/') {
            let level_key = LevelKey::from_str(path);
            if !current_node.children.contains_key(path) {
                // 子树中没有这个路径，添加进来
                let new_node = SubscriptionNode::new(level_key);
                current_node.children.insert(path.into(), new_node);
            }
            // 前进一步
            current_node = current_node.children.get_mut(path).unwrap();
        }

        current_node.data.insert(token, data);
    }

    /// 查找子树中和 topic 匹配的 filter
    fn matches<'a, I>(&self, mut topic_iter: I) -> Vec<&T>
    where
        I: Iterator<Item = &'a str> + Clone,
    {
        // 如果当前为叶子，直接返回数据
        if self.children.is_empty() {
            return self.data.values().collect();
        }

        match topic_iter.next() {
            Some(path) => {
                let mut matches = Vec::new();

                for (filter, node) in self.children.iter() {
                    if LevelKey::from_str(filter).matches(path) {
                        let datas = node.matches(topic_iter.clone());
                        matches.extend(datas);
                    }
                }

                matches
            }
            None => {
                // topic 路径不够了，如果当前节点有数据，返回
                self.data.values().collect()
            }
        }
    }

    /// 删除子树中对应的订阅
    fn remove<'a, I>(&mut self, mut filter_iter: I, token: u64)
    where
        I: Iterator<Item = &'a str> + Clone,
    {
        match filter_iter.next() {
            // 有下一个，去子树里找
            Some(path) => {
                // 子节点中包含此路径，执行删除操作
                if let Some(node) = self.children.get_mut(path) {
                    node.remove(filter_iter, token);
                    // 子节点删除后，如果子节点成为了叶子节点且数据为空，则删除这个子节点
                    if node.children.is_empty() && node.data.is_empty() {
                        self.children.remove(path);
                    }
                }
            }
            // 没有下一个，截止到当前节点
            None => {
                self.data.remove(&token);
            }
        }
    }
}

/// 订阅的filter的每一层
#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize)]
pub enum LevelKey {
    Concrete(String),
    SingleLevelWildcard,
    MultiLevelWildcard,
}

impl LevelKey {
    fn matches(&self, path: &str) -> bool {
        match self {
            LevelKey::Concrete(s) => path == s,
            LevelKey::SingleLevelWildcard | LevelKey::MultiLevelWildcard => true,
        }
    }

    fn from_str(path: &str) -> Self {
        match path {
            "+" => LevelKey::SingleLevelWildcard,
            "#" => LevelKey::MultiLevelWildcard,
            s => LevelKey::Concrete(s.into()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sub_tree_works() {
        let mut sub_tree = SubscriptionTree::new();

        // insert
        sub_tree.insert("iot/pid/dn/temperature", "test1");
        sub_tree.insert("iot/pid/+/temperature", "test1");
        sub_tree.insert("iot/+/dn/temperature", "test1");
        sub_tree.insert("iot/pid/dn/+", "test1");
        sub_tree.insert("iot/pid/dn/+", "test2");

        // search
        assert!(sub_tree.matches("iot/pid").is_empty());
        assert_eq!(sub_tree.matches("iot/pid/dn/temperature").len(), 5);
        assert_eq!(sub_tree.matches("iot/pid/dn/pressure").len(), 2);

        // remove
        sub_tree.remove("iot/pid/dn/temperature", 0);
        assert_eq!(sub_tree.matches("iot/pid/dn/temperature").len(), 4);
        sub_tree.remove("iot/pid/dn/+", 4);
        assert_eq!(sub_tree.matches("iot/pid/dn/temperature").len(), 3)
    }
}
