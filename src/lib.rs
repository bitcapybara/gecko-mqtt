#![allow(dead_code)]

//! 一个 mqtt 服务端库，用户可以使用此库构建自己的 mqtt broker

use std::result::Result;

use async_trait::async_trait;

pub mod broker;
pub mod config;
pub mod error;
mod network;
pub mod packet;
mod protocol;
mod cluster;

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
