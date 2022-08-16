pub(crate) use client::ClientConnection;
pub(crate) use peer::PeerConnection;
use tokio::io;

use super::{packet, v4::PacketType};

mod client;
mod peer;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("First packet not connect")]
    FirstPacketNotConnect,
    #[error("Packet error: {0}")]
    Packet(#[from] packet::Error),
    #[error("I/O: {0}")]
    IO(#[from] io::Error),
    #[error("Unexpected incoming message: {0:?}")]
    UnexpectedImcoming(PacketType),
}
