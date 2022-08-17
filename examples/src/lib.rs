#![allow(dead_code)]

use tokio::{fs, io::AsyncReadExt};

#[derive(Debug, serde::Deserialize)]
pub struct Config {
    pub broker: Broker,
}

#[derive(Debug, serde::Deserialize)]
pub struct Broker {
    pub listen: Listen,
}

#[derive(Debug, serde::Deserialize)]
pub struct Listen {
    pub client: String,
    pub peer: String,
}

impl Config {
    pub async fn from_path(path: &str) -> Self {
        let mut file = fs::File::open(path).await.unwrap();
        let mut s = String::new();
        file.read_to_string(&mut s).await.unwrap();

        toml::from_str::<Config>(&s).unwrap()
    }
}
