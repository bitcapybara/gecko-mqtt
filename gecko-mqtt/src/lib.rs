#![allow(dead_code)]

//! 一个 mqtt 服务端库，用户可以使用此库构建自己的 mqtt broker

use async_trait::async_trait;
use network::v4::Login;

pub mod broker;
mod cluster;
pub mod error;
mod network;
mod protocol;
mod server;

/// mqtt事件发生时的回调，由用户实现
///
#[async_trait]
pub trait Hook: Send + Sync + 'static {
    /// 客户端认证
    async fn authenticate(&self, login: Option<Login>) -> bool;
    /// 客户端上线
    async fn connected(&self, client_id: &str);
    /// 客户端连接断开
    async fn disconnect(&self, client_id: &str);
}

pub struct HookNoop;

#[async_trait]
impl Hook for HookNoop {
    /// 客户端认证
    async fn authenticate(&self, _login: Option<Login>) -> bool {
        true
    }
    /// 客户端上线
    async fn connected(&self, _client_id: &str) {}
    /// 客户端连接断开
    async fn disconnect(&self, _client_id: &str) {}
}
