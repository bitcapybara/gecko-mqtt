//! 网络层
//! 本层只关心网络读写优化，不包含任何协议相关逻辑

use std::sync::Arc;

pub(crate) use conn::{ClientConnection, PeerConnection};
pub(crate) use packet::v4;
use tokio::{
    net::TcpStream,
    select,
    sync::mpsc::{self, error::SendError, Receiver, Sender},
};

use crate::{
    network,
    protocol::{Incoming, Outgoing},
    Hook,
};

use self::v4::connack;

pub(crate) mod conn;
mod packet;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Unexpected router message")]
    UnexpectedRouterMessage,
    #[error("Connection error: {0}")]
    Connection(#[from] conn::Error),
    #[error("Packet error: {0}")]
    Packet(#[from] packet::Error),
    #[error("First connect fail")]
    FirstConnectFailed(connack::ConnectReturnCode),
    #[error("Send message to router error: {0}")]
    Send(#[from] SendError<Incoming>),
}

pub struct ClientEventLoop<H: Hook> {
    conn: ClientConnection,
    router_tx: Sender<Incoming>,
    _hook: Option<Arc<H>>,
    conn_rx: Receiver<Outgoing>,
}

impl<H: Hook> ClientEventLoop<H> {
    pub(crate) async fn new(
        stream: TcpStream,
        router_tx: Sender<Incoming>,
        _hook: Option<Arc<H>>,
    ) -> Result<Self, Error> {
        let mut conn = ClientConnection::new(stream);

        // conn_tx 由 router/session 持有，用于给当前这个 connection 发送消息
        let (conn_tx, mut conn_rx) = mpsc::channel(1000);

        // TODO 通过超时来处理 keepalive 逻辑
        // 第一个报文，必须是 connect 报文
        let connect = conn.read_connect().await?;
        // 发送给 router 处理
        router_tx
            .send(Incoming::Connect {
                packet: connect,
                conn_tx,
            })
            .await
            .unwrap();
        // 获取 router 处理结果
        let outcoming = conn_rx.recv().await.unwrap();
        let (_id, ack) = match outcoming {
            Outgoing::ConnAck { id, packet } => (id, packet),
            _ => return Err(Error::UnexpectedRouterMessage),
        };
        let return_code = ack.code;
        // 发送给客户端
        conn.write_connack(ack).await?;
        match return_code {
            // router 处理成功，开启循环
            connack::ConnectReturnCode::Success => Ok(Self {
                conn,
                router_tx,
                _hook,
                conn_rx,
            }),
            // 返回失败结果，退出循环
            code => Err(Error::FirstConnectFailed(code)),
        }
    }

    /// 开启事件循环
    /// * connect 报文已在 new 方法中处理过，这里如果收到 connect 报文，视为非法连接
    /// * 从 conn socket 网络层获取 packet 数据，发送给 router
    /// * 接收 router 的回复，写入 conn socket 网络层

    pub(crate) async fn start(mut self) -> Result<(), Error> {
        loop {
            select! {
                // 从网络层读数据
                res = self.conn.read_packets() => {
                    match res {
                        Ok(packets) => self.router_tx.send(Incoming::Data(packets)).await?,
                        Err(e) => return Err(network::Error::Connection(e)),
                    }
                }
                // 从 router 读回复
                res = self.conn_rx.recv() => {
                    match res {
                        Some(outgoing) => match outgoing {
                            Outgoing::Data(packet) => self.conn.write_packet(packet).await?,
                            _ => return Err(Error::UnexpectedRouterMessage)
                        },
                        None => todo!(),
                    }
                }
            }
        }
    }
}
