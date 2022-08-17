//! 分布层
//! 专注于集群功能的实现，如节点注册，节点地址获取，节点健康检查等
//! 当前有三种方式：
//! * 单机版本
//! * 少量节点：使用 raft 算法
//! * 多节点：借助第三方一致性存储，如 Etcd
//! * Etcd 模式，数据在本地没有备份，可能会增加访问数据的延迟

// pub(crate) use connection::Connection;
pub(crate) use dispatcher::Dispatcher;

mod dispatcher;
mod manager;
mod storage;
