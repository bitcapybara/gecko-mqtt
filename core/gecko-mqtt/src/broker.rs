use log::error;
use tokio::{net::TcpListener, sync::mpsc};

use crate::{
    error::Result,
    network::{self, ClientConnection},
    protocol::{ConnectionEventLoop, Router},
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

    async fn start<H: Hook>(&self, hook: H) -> Result<()> {
        // router 后台协程
        let (router_tx, router_rx) = mpsc::channel(1000);
        tokio::spawn(async move {
            let router = Router::new(hook, router_rx);
            if let Err(e) = router.start().await {
                error!("router exit error: {:?}", e)
            }
        });

        // TODO 开启 grpc server 和 PeerConnection 事件循环

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
            let router_tx = router_tx.clone();
            tokio::spawn(async move {
                let conn = ClientConnection::new(stream);
                if let Err(e) =
                    ConnectionEventLoop::start(network::Connection::Client(conn), router_tx.clone())
                        .await
                {
                    error!("eventloop on conn {0} exit error: {1:?}", addr, e)
                }
            });
        }
    }
}