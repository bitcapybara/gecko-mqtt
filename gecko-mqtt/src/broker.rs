use std::sync::Arc;

use futures::{TryFutureExt, FutureExt};
use log::{debug, error, info};
use tokio::{
    net::TcpListener,
    sync::mpsc::{self, Sender},
};

use crate::{
    config::Config,
    network::{conn, ClientEventLoop, PeerConnection},
    protocol::{router, Incoming, Router},
    server::PeerServer,
    Hook, HookNoop,
};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Router Error: {0}")]
    Router(#[from] router::Error),
    #[error("Grpc Error: {0}")]
    Grpc(#[from] tonic::transport::Error),
    #[error("Peer conn error: {0}")]
    PeerConn(#[from] conn::Error),
}

/// 代表一个 mqtts 节点
pub struct Broker {
    cfg: Config,
}

impl Broker {
    pub fn new(cfg: Config) -> Self {
        Self { cfg }
    }

    pub async fn start(self) -> Result<(), Error> {
        self.start_with_hook(Arc::new(HookNoop)).await
    }

    pub async fn start_with_hook(self, hook: Arc<impl Hook>) -> Result<(), Error> {
        // router 后台协程
        let (router_tx, router_rx) = mpsc::channel(1000);
        let router_hook = hook.clone();
        let session_cfg = self.cfg.session.clone();

        debug!("start router loop");
        let router = Router::new(session_cfg, router_hook, router_rx);
        let (router_task, router_handle) = router.start().map_err(Error::Router).remote_handle();
        tokio::spawn(router_task);

        // 开启 grpc peer server
        let (peer_tx, peer_rx) = mpsc::channel(1000);
        let grpc_addr = self.cfg.broker.peer_addr.parse().unwrap();
        debug!("start peer server loop");
        let (grpc_task, grpc_handle) = tonic::transport::Server::builder()
            .add_service(PeerServer::new_server(peer_tx))
            .serve(grpc_addr)
            .map_err(Error::Grpc).remote_handle();
        tokio::spawn(grpc_task);

        // 开启 peer conn 事件循环
        debug!("start peer conn event loop");
        let peer = PeerConnection::new(peer_rx);
        let (peer_task, peer_handle) = peer.start(router_tx.clone()).map_err(Error::PeerConn).remote_handle();
        tokio::spawn(peer_task);

        // 开启客户端连接监听
        debug!("start client server loop");
        let (tcp_task, tcp_handle) = self.start_tcp(router_tx, hook).remote_handle();
        tokio::spawn(tcp_task);

        tokio::try_join!(router_handle, grpc_handle, peer_handle, tcp_handle)?;

        Ok(())
    }

    async fn start_tcp(
        self,
        router_tx: Sender<Incoming>,
        hook: Arc<impl Hook>,
    ) -> Result<(), Error> {
        let listener = TcpListener::bind(&self.cfg.broker.client_addr).await.unwrap();
        loop {
            // 获取到连接
            let (stream, addr) = match listener.accept().await {
                Ok((s, a)) => (s, a),
                Err(_) => {
                    log::error!("accept tcp stream err");
                    continue;
                }
            };
            info!("new stream comming in: {}", addr);

            // 事件循环
            let client_router_tx = router_tx.clone();
            let client_hook = hook.clone();
            tokio::spawn(async move {
                match ClientEventLoop::new(stream, client_router_tx.clone(), client_hook).await {
                    Ok(event_loop) => {
                        let client_id = event_loop.client_id.clone();
                        if let Err(e) = event_loop.start().await {
                            if let Err(e) = client_router_tx
                                .send(Incoming::Disconnect {
                                    client_id: client_id.clone(),
                                })
                                .await
                            {
                                error!("send disconnect to router channel error {:#}", e);
                            }
                            error!("eventloop on client {0} exit error: {1:#}", client_id, e)
                        }
                    }
                    Err(e) => {
                        error!("eventloop read first connect packet err: {:#}", e)
                    }
                }
            });
        }
    }
}
