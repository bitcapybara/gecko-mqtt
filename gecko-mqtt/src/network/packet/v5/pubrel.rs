use super::PacketProperties;

#[derive(Debug)]
pub struct PubRel {
    pub packet_id: u16,
    pub reason: PubRelReason,
    pub properties: Option<PacketProperties>,
}

#[derive(Debug)]
#[repr(u8)]
pub enum PubRelReason {
    Success = 0,
    PacketIdentifierNotFound = 146,
}
