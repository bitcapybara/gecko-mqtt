//! 协议层
//! 处理协议相关的逻辑，依赖于底层的网络层进行网络读写

use tokio::{
    select,
    sync::mpsc::{self, error::SendError, Receiver, Sender},
};

use crate::network::{
    self,
    v4::{connack, ConnAck, Connect, Packet, PacketType},
};

pub(crate) use router::Router;
pub use session::SessionState;

mod router;
mod session;

pub(crate) type ConnectionId = usize;

#[derive(Debug, thiserror::Error)]
pub(crate) enum Error {
    #[error("Read connection error: {0}")]
    ReadConnection(#[from] network::conn::Error),
    #[error("Unexpected router message")]
    UnexpectedRouterMessage,
    #[error("First connect fail")]
    FirstConnectFailed(connack::ConnectReturnCode),
    #[error("Unexpected incoming message: {0:?}")]
    UnexpectedImcoming(PacketType),
    #[error("Send message to router error: {0}")]
    Send(#[from] SendError<Incoming>),
}

/// 发送给 router 的消息
#[derive(Debug)]
pub(crate) enum Incoming {
    Connect {
        packet: Connect,
        conn_tx: Sender<Outgoing>,
    },
    Data(Vec<Packet>),
}

/// router 发送给客户端的回复
pub(crate) enum Outgoing {
    ConnAck { id: ConnectionId, packet: ConnAck },
    Data(Packet),
}

/// 一个客户端或对等节点连接的**事件循环**
pub(crate) struct ConnectionEventLoop {
    /// 连接 id
    id: ConnectionId,
    /// 给 router 发送消息的管道
    router_tx: Sender<Incoming>,
    /// 从协议层接收到的消息
    conn_rx: Receiver<Outgoing>,
}

impl ConnectionEventLoop {
    /// 开启事件循环
    /// * connect 报文已在 new 方法中处理过，这里如果收到 connect 报文，视为非法连接
    /// * 从 conn socket 网络层获取 packet 数据，发送给 router
    /// * 接收 router 的回复，写入 conn socket 网络层
    pub(crate) async fn start(
        conn: network::Connection,
        router_tx: Sender<Incoming>,
    ) -> Result<(), Error> {
        // conn_tx 由 router/session 持有，用于给当前这个 connection 发送消息
        let (conn_tx, mut conn_rx) = mpsc::channel(1000);
        match conn {
            network::Connection::Client(mut cc) => {
                // TODO 超时处理
                // 第一个报文，必须是 connect 报文
                let connect = cc.read_connect().await?;
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
                let (id, ack) = match outcoming {
                    Outgoing::ConnAck { id, packet } => (id, packet),
                    _ => return Err(Error::UnexpectedRouterMessage),
                };
                let return_code = ack.code;
                // 发送给客户端
                cc.write_connack(ack).await?;
                match return_code {
                    // router 处理成功，开启循环
                    connack::ConnectReturnCode::Success => {
                        ConnectionEventLoop {
                            id,
                            router_tx,
                            conn_rx,
                        }
                        .start_client_conn(cc)
                        .await?
                    }
                    // 返回失败结果，退出循环
                    code => return Err(Error::FirstConnectFailed(code)),
                }
            }
            network::Connection::Peer(pc) => {
                ConnectionEventLoop {
                    id: 0,
                    router_tx,
                    conn_rx,
                }
                .start_peer_conn(pc)
                .await?
            }
        }

        Ok(())
    }

    async fn start_client_conn(
        &mut self,
        mut conn: network::ClientConnection,
    ) -> Result<(), Error> {
        loop {
            select! {
                // 从网络层读数据
                res = conn.read_packets() => {
                    match res {
                        Ok(packets) => self.router_tx.send(Incoming::Data(packets)).await?,
                        Err(e) => return Err(Error::ReadConnection(e)),
                    }
                }
                // 从 router 读回复
                res = self.conn_rx.recv() => {
                    match res {
                        Some(outgoing) => match outgoing {
                            Outgoing::Data(packet) => conn.write_packet(packet).await?,
                            _ => return Err(Error::UnexpectedRouterMessage)
                        },
                        None => todo!(),
                    }

                }
            }
        }
    }

    async fn start_peer_conn(&self, _conn: network::PeerConnection) -> Result<(), Error> {
        todo!()
    }
}
