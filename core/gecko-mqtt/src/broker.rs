use std::sync::Arc;

use log::error;
use tokio::{net::TcpListener, sync::mpsc};

use crate::{
    error::Result,
    network::{ClientConnection, PeerConnection},
    protocol::Router,
    server::PeerServer,
    Hook,
};

pub struct BrokerConfig {
    pub addr: String,
}

/// 代表一个 mqtts 节点
pub struct Broker {
    // 客户端监听地址
    tcp_addr: String,
    // 对等节点监听地址
    grpc_addr: String,
}

impl Broker {
    fn new(_cfg: BrokerConfig) -> Result<Self> {
        todo!()
    }

    async fn start<H: Hook>(&self, hook: Arc<H>) -> Result<()> {
        // router 后台协程
        let (router_tx, router_rx) = mpsc::channel(1000);
        let router_hook = hook.clone();
        tokio::spawn(async move {
            let router = Router::new(router_hook, router_rx);
            if let Err(e) = router.start().await {
                error!("router exit error: {:?}", e)
            }
        });

        // 开启 grpc peer server
        let (peer_tx, peer_rx) = mpsc::channel(1000);
        let grpc_addr = self.grpc_addr.parse().unwrap();
        tokio::spawn(async move {
            tonic::transport::Server::builder()
                .add_service(PeerServer::new_server(peer_tx))
                .serve(grpc_addr)
                .await
                .unwrap();
        });

        // 开启 peer conn 事件循环
        let peer_router_tx = router_tx.clone();
        tokio::spawn(async move {
            let conn = PeerConnection::new(peer_rx);
            if let Err(e) = conn.start(peer_router_tx) {
                error!("eventloop on peer conn exit error: {0}", e)
            }
        });

        // 开启客户端连接监听
        let listener = TcpListener::bind(&self.tcp_addr).await.unwrap();

        loop {
            // 获取到连接
            let (stream, addr) = match listener.accept().await {
                Ok((s, a)) => (s, a),
                Err(_) => {
                    log::error!("accept tcp stream err");
                    continue;
                }
            };
            log::info!("new stream comming in: {}", addr);

            // 事件循环
            let client_router_tx = router_tx.clone();
            let client_hook = hook.clone();
            tokio::spawn(async move {
                let conn = ClientConnection::new(stream);
                if let Err(e) = conn.start(client_router_tx, client_hook).await {
                    error!("eventloop on conn {0} exit error: {1:?}", addr, e)
                }
            });
        }
    }
}
