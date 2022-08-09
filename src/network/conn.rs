pub(crate) use client::ClientConnection;
pub(crate) use peer::PeerConnection;

mod client;
mod peer;

pub(crate) enum Connection {
    Client(ClientConnection),
    Peer(PeerConnection)
}