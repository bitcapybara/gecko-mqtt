use crate::{error::Result, packet::Packet};

/// 节点之间的连接
pub(crate) struct PeerConnection {}

impl PeerConnection {
    pub(crate) async fn read_packets(&mut self) -> Result<Vec<Packet>> {
        todo!()
    }
}
