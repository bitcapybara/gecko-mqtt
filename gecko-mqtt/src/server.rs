use gecko_mqtt_proto::{
    gecko_peer_server::{GeckoPeer, GeckoPeerServer},
    TransferPacketRequest, TransferPacketResponse,
};
use tokio::sync::mpsc::Sender;

pub(crate) struct PeerServer {
    peer_tx: Sender<()>,
}

impl PeerServer {
    pub(crate) fn new_server(peer_tx: Sender<()>) -> GeckoPeerServer<PeerServer> {
        GeckoPeerServer::new(Self { peer_tx })
    }
}

#[tonic::async_trait]
impl GeckoPeer for PeerServer {
    async fn transfer_packet(
        &self,
        _request: tonic::Request<TransferPacketRequest>,
    ) -> Result<tonic::Response<TransferPacketResponse>, tonic::Status> {
        todo!()
    }
}
