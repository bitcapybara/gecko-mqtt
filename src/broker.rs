use tokio::net::TcpListener;

use crate::error::Result;

pub struct BrokerConfig {
    pub addr: String,
}

/// 代表一个 gecko 节点
pub struct Broker {
    addr: String,
}

impl Broker {
    fn new(_cfg: BrokerConfig) -> Result<Self> {
        todo!()
    }

    async fn start(&self) -> Result<()> {
        let listener = TcpListener::bind(&self.addr).await.unwrap();

        loop {
            // 获取到连接
            let (_stream, addr) = match listener.accept().await {
                Ok((s, a)) => (s, a),
                Err(_) => {
                    log::error!("accept tcp stream err");
                    continue;
                }
            };
            log::info!("new stream comming in: {}", addr)

            // 
        }
    }
}
