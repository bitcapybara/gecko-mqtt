//! 网络层
//! 本层只关心网络读写优化，不包含任何协议相关逻辑

pub(crate) use conn::Connection;
pub(crate) use packet::v4;

mod conn;
mod packet;
