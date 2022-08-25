use tokio::{fs, io::AsyncReadExt};

#[derive(Debug, serde::Deserialize)]
pub struct Config {
    pub broker: Broker,
    pub session: Session,
}

#[derive(Debug, serde::Deserialize)]
pub struct Broker {
    pub client_addr: String,
    pub peer_addr: String,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct Session {
    #[serde(default)]
    pub expire_interval: Option<u64>,
}

impl Config {
    pub async fn from_path(path: &str) -> Self {
        let mut file = fs::File::open(path).await.unwrap();
        let mut s = String::new();
        file.read_to_string(&mut s).await.unwrap();

        toml::from_str::<Config>(&s).unwrap()
    }
}
