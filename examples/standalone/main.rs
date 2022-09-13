use std::sync::Arc;

use async_trait::async_trait;
use clap::Parser;
use flexi_logger::{colored_opt_format, Logger};
use gecko_mqtt::config::Config;
use gecko_mqtt::{broker, Hook, Login};
use log::info;

#[derive(Debug, serde::Deserialize, clap::Parser)]
struct Args {
    #[serde(default)]
    #[clap(long, value_parser)]
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
    let envs = envy::from_env::<Args>().unwrap();
    let config_file_env = envs.config_file;

    // 命令行
    let args = Args::parse();
    let config_file = config_file_env.unwrap_or_else(|| args.config_file.unwrap());

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
