pub(crate) use client::ClientConnection;
pub(crate) use peer::PeerConnection;
use tokio::io;

use super::packet;

mod client;
mod peer;

#[derive(Debug, thiserror::Error)]
pub(crate) enum Error {
    #[error("First packet not connect")]
    FirstPacketNotConnect,
    #[error("Packet error: {0}")]
    Packet(#[from] packet::Error),
    #[error("I/O: {0}")]
    IO(#[from] io::Error),
}

pub(crate) enum Connection {
    Client(ClientConnection),
    Peer(PeerConnection),
}
