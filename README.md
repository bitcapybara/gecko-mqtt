# gecko-mqtt

使用 rust 语言实现的 mqtt broker 库，实现参考了 [emqx](https://github.com/emqx/emqx), [rumqtt](https://github.com/bytebeamio/rumqtt) 和 [mqtt-broker](https://github.com/bschwind/mqtt-broker)

当前已实现的单机内存版本broker，协议版本为 MQTT 3.1.1

内部代码设计，参考 [design.md](design.md)

## 使用

用户可以依赖本项目构建自己的 mqtt broker，具体使用可参考 examples 目录下的代码。

### 基本使用

```rust
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
```

### 自定义回调

```rust
use std::sync::Arc;

use async_trait::async_trait;
use flexi_logger::{colored_opt_format, Logger};
use gecko_mqtt::{broker, Hook, Login};
use gecko_mqtt::config::Config;
use log::info;

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
    broker::Broker::new(cfg).start_with_hook(Arc::new(CustomHook)).await.unwrap()
}

struct CustomHook;

#[async_trait]
impl Hook for CustomHook {
    /// 客户端认证
    async fn authenticate(&self, _login: Option<Login>) -> bool {
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
```

### 启动

```bash
cargo run --bin standalone
```

## TODO

- [x] 单机内存，协议版本 v3.1.1（需要更多测试）
- [ ] 单机内存，协议版本 v5
- [ ] 单机持久化
- [ ] 分布式持久化