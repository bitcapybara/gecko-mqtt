use super::PacketProperties;

#[derive(Debug)]
pub struct PubComp {
    pub pkid: u16,
    pub reason: PubCompReason,
    pub properties: Option<PacketProperties>,
}

#[derive(Debug)]
#[repr(u8)]
pub enum PubCompReason {
    Success = 0,
    PacketIdentifierNotFound = 146,
}
