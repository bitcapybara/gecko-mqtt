use std::sync::Arc;

use log::{debug, error, info};
use tokio::{net::TcpListener, sync::mpsc};

use crate::{
    config::Config,
    error::Error,
    network::{ClientEventLoop, PeerConnection},
    protocol::{Incoming, Router},
    server::PeerServer,
    Hook, HookNoop,
};

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
        tokio::spawn(async move {
            debug!("start router loop");
            let router = Router::new(session_cfg, router_hook, router_rx);
            if let Err(e) = router.start().await {
                error!("router exit error: {:?}", e)
            }
        });

        // 开启 grpc peer server
        let (peer_tx, peer_rx) = mpsc::channel(1000);
        let grpc_addr = self.cfg.broker.peer_addr.parse().unwrap();
        tokio::spawn(async move {
            debug!("start peer server loop");
            tonic::transport::Server::builder()
                .add_service(PeerServer::new_server(peer_tx))
                .serve(grpc_addr)
                .await
                .unwrap();
        });

        // 开启 peer conn 事件循环
        let peer_router_tx = router_tx.clone();
        tokio::spawn(async move {
            debug!("start peer conn event loop");
            let conn = PeerConnection::new(peer_rx);
            if let Err(e) = conn.start(peer_router_tx) {
                error!("eventloop on peer conn exit error: {:#}", e)
            }
        });

        // 开启客户端连接监听
        let listener = TcpListener::bind(&self.cfg.broker.client_addr)
            .await
            .unwrap();
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
