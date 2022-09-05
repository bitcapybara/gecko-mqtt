use super::PacketProperties;

#[derive(Debug)]
pub struct PubAck {
    pub packet_id: u16,
    pub reason: PubAckReason,
    pub properties: Option<PacketProperties>,
}

#[derive(Debug)]
#[repr(u8)]
pub enum PubAckReason {
    Success = 0,
    NoMatchingSubscribers = 16,
    UnspecifiedError = 128,
    ImplementationSpecificError = 131,
    NotAuthorized = 135,
    TopicNameInvalid = 144,
    PacketIdentifierInUse = 145,
    QuotaExceeded = 151,
    PayloadFormatInvalid = 153,
}
