use crate::network;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Network conn error: {0}")]
    Network(#[from] network::Error),
}
