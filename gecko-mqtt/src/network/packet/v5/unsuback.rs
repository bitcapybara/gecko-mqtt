use super::PacketProperties;

#[derive(Debug)]
pub struct UnsubAck {
    pub packet_id: u16,
    pub reasons: Vec<UnsubAckReason>,
    pub properties: Option<PacketProperties>,
}

#[derive(Debug)]
#[repr(u8)]
pub enum UnsubAckReason {
    Success = 0x00,
    NoSubscriptionExisted = 0x11,
    UnspecifiedError = 0x80,
    ImplementationSpecificError = 0x83,
    NotAuthorized = 0x87,
    TopicFilterInvalid = 0x8F,
    PacketIdentifierInUse = 0x91,
}
