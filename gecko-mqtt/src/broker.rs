use std::sync::Arc;

use futures::TryFutureExt;
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
    #[error("Conn error: {0}")]
    Conn(#[from] conn::Error),
}

/// 代表一个 mqtts 节点
pub struct Broker {
    cfg: Config,
}

impl Broker {
    pub fn new(cfg: Config) -> Self {
        Self { cfg }
    }

    pub async fn start(&self) -> Result<(), Error> {
        self.start_with_hook(Arc::new(HookNoop)).await
    }

    pub async fn start_with_hook<H: Hook>(&self, hook: Arc<H>) -> Result<(), Error> {
        // router 后台协程
        let (router_tx, router_rx) = mpsc::channel(1000);
        let router_hook = hook.clone();
        let session_cfg = self.cfg.session.clone();

        debug!("start router loop");
        let router = Router::new(session_cfg, router_hook, router_rx);
        let router_handle = router.start().map_err(Error::Router);

        // 开启 grpc peer server
        let (peer_tx, peer_rx) = mpsc::channel(1000);
        let grpc_addr = self.cfg.broker.peer_addr.parse().unwrap();
        debug!("start peer server loop");
        let grpc_handle = tonic::transport::Server::builder()
            .add_service(PeerServer::new_server(peer_tx))
            .serve(grpc_addr)
            .map_err(Error::Grpc);

        // 开启 peer conn 事件循环
        debug!("start peer conn event loop");
        let conn = PeerConnection::new(peer_rx);
        let peer_handle = conn.start(router_tx.clone()).map_err(Error::Conn);

        // 开启客户端连接监听
        let tcp_handle = Self::start_tcp(&self.cfg.broker.client_addr, router_tx, hook);

        tokio::try_join!(router_handle, grpc_handle, peer_handle, tcp_handle)?;

        Ok(())
    }

    async fn start_tcp<H: Hook>(
        addr: &str,
        router_tx: Sender<Incoming>,
        hook: Arc<H>,
    ) -> Result<(), Error> {
        let listener = TcpListener::bind(addr).await.unwrap();
        debug!("start client server loop");
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
