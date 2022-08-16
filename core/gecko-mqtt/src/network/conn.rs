pub(crate) use client::ClientConnection;
pub(crate) use peer::PeerConnection;
use tokio::{io, sync::mpsc::error::SendError};

use crate::protocol::Incoming;

use super::{
    packet,
    v4::{connack, PacketType},
};

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
    #[error("Unexpected router message")]
    UnexpectedRouterMessage,
    #[error("First connect fail")]
    FirstConnectFailed(connack::ConnectReturnCode),
    #[error("Unexpected incoming message: {0:?}")]
    UnexpectedImcoming(PacketType),
    #[error("Send message to router error: {0}")]
    Send(#[from] SendError<Incoming>),
}
