use super::PacketProperties;

#[derive(Debug)]
pub struct SubAck {
    pub packet_id: u16,
    pub return_codes: Vec<SubscribeReasonCode>,
    pub properties: Option<PacketProperties>,
}

#[derive(Debug)]
#[repr(u8)]
pub enum SubscribeReasonCode {
    QoS0 = 0,
    QoS1 = 1,
    QoS2 = 2,
    Unspecified = 128,
    ImplementationSpecific = 131,
    NotAuthorized = 135,
    TopicFilterInvalid = 143,
    PkidInUse = 145,
    QuotaExceeded = 151,
    SharedSubscriptionsNotSupported = 158,
    SubscriptionIdNotSupported = 161,
    WildcardSubscriptionsNotSupported = 162,
}
