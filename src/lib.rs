#![allow(dead_code)]

//! 一个 mqtt 服务端库，用户可以使用此库构建自己的 mqtt broker

use async_trait::async_trait;

use error::Result;

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
    fn start(&self) -> Result<()>;
    fn close(&self) -> Result<()>;

    fn get(&self, key: Vec<u8>) -> Result<Vec<u8>>;
    fn set(&self, key: Vec<u8>, value: Vec<u8>) -> Result<()>;
    fn del(&self, key: Vec<u8>) -> Result<()>;
}

/// mqtt事件发生时的回调，由用户实现
#[async_trait]
pub trait Hook {
    async fn on_connect() -> Result<()>;
}
