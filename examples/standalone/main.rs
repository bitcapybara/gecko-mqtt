use std::sync::Arc;

use async_trait::async_trait;
use flexi_logger::{colored_opt_format, Logger};
use gecko_mqtt::config::Config;
use gecko_mqtt::{broker, Hook, Login};
use log::info;

#[derive(Debug, serde::Deserialize)]
struct Args {
    #[serde(default)]
    config_file: Option<String>,
}

#[tokio::main]
async fn main() {
    // 日志
    Logger::try_with_str("debug")
        .unwrap()
        .format(colored_opt_format)
        .start()
        .unwrap();

    // 环境变量
    let args = envy::from_env::<Args>().unwrap();
    let config_file = args.config_file.unwrap_or_else(|| "standalone.toml".into());

    // 获取配置
    let cfg = Config::from_path(&config_file).await;

    // 启动 broker
    broker::Broker::new(cfg)
        .start_with_hook(Arc::new(CustomHook))
        .await
        .unwrap()
}

struct CustomHook;

#[async_trait]
impl Hook for CustomHook {
    /// 客户端认证
    async fn authenticate(&self, _login: Login) -> bool {
        info!("login authenticate");
        true
    }
    /// 客户端上线
    async fn connected(&self, client_id: &str) {
        info!("client {0} connected", client_id)
    }
    /// 客户端连接断开
    async fn disconnect(&self, client_id: &str) {
        info!("client {0} disconnect", client_id)
    }
}
