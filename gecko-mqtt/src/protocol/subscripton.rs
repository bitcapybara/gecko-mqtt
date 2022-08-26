use std::{collections::HashMap, str::FromStr};

use crate::network::topic::topic_has_wildcards;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Parse topic level error")]
    ParseFilterLevel,
}

pub struct SubscriptionTree<T> {
    /// 订阅树的根节点，是个空节点
    root: SubscriptionNode<T>,
    /// 插入的每一个数据，分配一个唯一的 token 号，方便查询和删除
    token: u64,
}

impl<T> SubscriptionTree<T> {
    pub fn new() -> Self {
        Self {
            root: SubscriptionNode::empty(),
            token: 0,
        }
    }

    /// 插入一个订阅记录
    pub fn insert(&mut self, filter: &str, data: T) -> Result<u64, Error> {
        let filter_path = filter.split('/');

        let token = self.token;
        self.root.add_node(filter_path, token, data)?;
        self.token += 1;

        Ok(token)
    }

    /// 查找发布消息的主题匹配的记录
    pub fn matches(&mut self, _topic: &str) -> Vec<&T> {
        vec![]
    }

    /// 删除订阅记录
    pub fn remove(&mut self, _filter: &str, _token: u64) {
        todo!()
    }
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

impl<T> SubscriptionNode<T> {
    fn empty() -> Self {
        Self {
            key: LevelKey::Concrete("".into()),
            data: HashMap::with_capacity(0),
            children: HashMap::new(),
        }
    }

    fn new(key: LevelKey) -> Self {
        Self {
            key,
            data: HashMap::new(),
            children: HashMap::new(),
        }
    }

    /// 将订阅加到当前节点的子树中
    fn add_node<'a, I>(&mut self, filter_iter: I, token: u64, data: T) -> Result<(), Error>
    where
        I: Iterator<Item = &'a str>,
    {
        let mut current_node = self;

        for path in filter_iter {
            if current_node.children.contains_key(path) {
                // 子树中有这个路径，前往下一个节点
                current_node = current_node.children.get_mut(path).unwrap();
            } else {
                // 子树中没有这个路径，添加进来
                let level_key = LevelKey::from_str(path)?;
                let new_node = SubscriptionNode::new(level_key);
                current_node.children.insert(path.into(), new_node);
            }
        }

        current_node.data.insert(token, data);

        Ok(())
    }
}

/// 订阅的filter的每一层
pub enum LevelKey {
    Concrete(String),
    SingleLevelWildcard,
    MultiLevelWildcard,
}

impl FromStr for LevelKey {
    type Err = Error;

    fn from_str(path: &str) -> Result<Self, Self::Err> {
        match path {
            "+" => Ok(LevelKey::SingleLevelWildcard),
            "#" => Ok(LevelKey::MultiLevelWildcard),
            s if topic_has_wildcards(s) => Err(Error::ParseFilterLevel),
            s => Ok(LevelKey::Concrete(s.into())),
        }
    }
}
