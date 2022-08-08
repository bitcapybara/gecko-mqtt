#![allow(dead_code)]

//! 一个 mqtt 服务端库，用户可以使用此库构建自己的 mqtt broker

use std::result::Result;

use async_trait::async_trait;
use protocol::SessionStore;

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
#[async_trait]
pub trait MetaStore {
    type Error;

    fn start(&self) -> Result<(), Self::Error>;
    fn close(&self) -> Result<(), Self::Error>;

    /// 添加一个新会话
    async fn add_new_session(session: SessionStore) -> Result<(), Self::Error>;
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
