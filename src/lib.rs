#![allow(dead_code)]

//! 一个 mqtt 服务端库，用户可以使用此库构建自己的 mqtt broker

use std::result::Result;

use async_trait::async_trait;
use protocol::SessionState;

pub mod broker;
pub mod config;
pub mod error;
mod network;
pub mod packet;
mod protocol;

/// 存储接口，可由用户指定实现
/// * 对于单节点部署，推荐使用本地嵌入式 kv 存储
/// * 对于少数节点组成的集群(3-5个节点)，推荐使用 raft 算法存储
/// * 对于多数节点组成的集群(5个以上)，推荐使用强一致性存储服务如 Etcd 等
///
/// **NOTE**:
/// 用于存储所有节点通用的元数据，不与特定节点耦合
#[async_trait]
pub trait MetaStore {
    type Error;

    async fn start(&self) -> Result<(), Self::Error>;
    async fn close(&self) -> Result<(), Self::Error>;

    /// 添加一个新会话
    async fn add_new_session(session: SessionState) -> Result<(), Self::Error>;
}

/// 集群管理接口，由用户实现
/// 存储和特定节点有关系的元数据，如路由表
/// 当使用 raft 算法时，天然集群功能
/// 当使用 etcd 存储时，由服务发现功能实现
#[async_trait]
pub trait Cluster {
    type Error;

    async fn is_leader() -> Result<bool, Self::Error>;
}

/// mqtt事件发生时的回调，由用户实现
#[async_trait]
pub trait Hook {
    type Error;

    /// 客户端认证
    async fn authenticate() -> Result<(), Self::Error>;
    /// 客户端上线
    async fn connected();
    /// 客户端连接断开
    async fn disconnect();
}
