use gecko_mqtt::{broker::{BrokerConfig, self}};
use gecko_mqtt_examples::Config;

#[tokio::main]
async fn main() {
    let cfg = Config::from_path("./standalone.toml").await;

    let broker = broker::Broker::new(BrokerConfig {
        client_listen_addr: cfg.broker.listen.client,
        peer_listen_addr: cfg.broker.listen.peer,
    });

    broker.start().await.unwrap()
}
