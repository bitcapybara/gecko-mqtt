//! 网络层
//! 本层只关心网络读写优化，不包含任何协议相关逻辑

use std::sync::Arc;

pub(crate) use conn::{ClientConnection, PeerConnection};
pub(crate) use packet::v4;

use tokio::{
    net::TcpStream,
    select,
    sync::mpsc::{self, error::SendError, Receiver, Sender},
    time,
};

use crate::{
    network,
    protocol::{Incoming, Outgoing},
    Hook,
};

use self::v4::{connack, ConnAck, ConnectReturnCode};

pub(crate) mod conn;
pub(crate) mod packet;
pub(crate) mod topic;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Unexpected router message")]
    UnexpectedRouterMessage,
    #[error("Connection error: {0}")]
    Connection(#[from] conn::Error),
    #[error("First connect fail")]
    FirstConnectFailed(connack::ConnectReturnCode),
    #[error("Send message to router error: {0}")]
    SendIncoming(#[from] SendError<Incoming>),
    #[error("Send message to conn self error: {0}")]
    SendOutgoing(#[from] SendError<Outgoing>),
}

pub struct ClientEventLoop<H: Hook> {
    client_id: String,
    conn: ClientConnection,
    router_tx: Sender<Incoming>,
    hook: Arc<H>,
    conn_tx: Sender<Outgoing>,
    conn_rx: Receiver<Outgoing>,
    keepalive: time::Duration,
}

impl<H: Hook> ClientEventLoop<H> {
    pub(crate) async fn new(
        stream: TcpStream,
        router_tx: Sender<Incoming>,
        hook: Arc<H>,
    ) -> Result<Self, Error> {
        let mut conn = ClientConnection::new(stream);

        // conn_tx 由 router/session 持有，用于给当前这个 connection 发送消息
        let (conn_tx, mut conn_rx) = mpsc::channel(1000);

        // 第一个报文，必须是 connect 报文
        let connect = conn.read_connect().await?;
        let client_id = connect.client_id.clone();
        // 调用回调，认证
        let login = hook.authenticate(connect.login.clone()).await;
        if !login {
            // If a server sends a CONNACK packet containing a non-zero return code it MUST set Session Present to 0 [MQTT-3.2.2-4].
            conn_tx
                .send(Outgoing::ConnAck(ConnAck::new(
                    ConnectReturnCode::NotAuthorized,
                    false,
                )))
                .await?;
        }
        let keep_alive = time::Duration::from_secs(connect.keep_alive as u64);
        // 发送给 router 处理
        router_tx
            .send(Incoming::Connect {
                connect,
                conn_tx: conn_tx.clone(),
            })
            .await
            .unwrap();
        // 获取 router 处理结果
        let outcoming = conn_rx.recv().await.unwrap();
        let ack = match outcoming {
            Outgoing::ConnAck(packet) => packet,
            _ => return Err(Error::UnexpectedRouterMessage),
        };
        let return_code = ack.code;
        // 发送给客户端
        conn.write_connack(ack).await?;
        // 调用回调，连接
        hook.connected(&client_id).await;
        match return_code {
            // router 处理成功，开启循环
            connack::ConnectReturnCode::Success => Ok(Self {
                client_id,
                conn,
                router_tx,
                hook,
                conn_tx,
                conn_rx,
                keepalive: keep_alive + keep_alive.mul_f32(0.5),
            }),
            // 返回失败结果，退出循环
            code => Err(Error::FirstConnectFailed(code)),
        }
    }

    /// 开启事件循环
    /// * connect 报文已在 new 方法中处理过，这里如果收到 connect 报文，视为非法连接
    /// * 从 conn socket 网络层获取 packet 数据，发送给 router
    /// * 接收 router 的回复，写入 conn socket 网络层
    /// TODO 增加 Disconnect 错误类型，遇到这种错误，需要退出事件循环，关闭网络连接
    pub(crate) async fn start(mut self) -> Result<(), Error> {
        loop {
            select! {
                // 从网络层读数据
                reads = self.conn.read_more(self.keepalive) => {
                    match reads {
                        Ok(packets) => {
                            self.router_tx.send(Incoming::Data{
                                client_id: self.client_id.clone(),
                                packets
                            }).await?;
                        },
                        Err(e) => return Err(network::Error::Connection(e)),
                    }
                }
                // 从 router 读回复
                recv = self.conn_rx.recv() => {
                    match recv {
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
