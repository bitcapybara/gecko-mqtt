use flexi_logger::{colored_opt_format, Logger};
use gecko_mqtt::broker;
use gecko_mqtt::config::Config;

#[tokio::main]
async fn main() {
    // 日志
    Logger::try_with_str("debug")
        .unwrap()
        .format(colored_opt_format)
        .start()
        .unwrap();

    // 获取配置
    let cfg = Config::from_path("./examples/config/standalone.toml").await;

    // 启动 broker
    broker::Broker::new(cfg).start().await.unwrap()
}
