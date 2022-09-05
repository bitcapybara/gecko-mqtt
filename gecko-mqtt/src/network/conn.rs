pub(crate) use client::ClientConnection;
pub(crate) use peer::PeerConnection;
use tokio::{io, time};

use super::packet::{self, PacketType};

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
    #[error("Keep alive timeout")]
    KeepAlive(#[from] time::error::Elapsed),
    #[error("Connection closed by peer")]
    ConnectionAborted,
    #[error("Connection reset by peer")]
    ConnectionReset,
}
