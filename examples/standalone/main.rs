use gecko_mqtt::broker::{self, BrokerConfig};
use examples::Config;

#[tokio::main]
async fn main() {
    // 获取配置
    let cfg = Config::from_path("./standalone.toml").await;

    // 启动 broker
    broker::Broker::new(BrokerConfig {
        client_listen_addr: cfg.broker.listen.client,
        peer_listen_addr: cfg.broker.listen.peer,
    })
    .start()
    .await
    .unwrap()
}
