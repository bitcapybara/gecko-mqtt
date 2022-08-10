pub mod v311;
pub mod v5;

#[derive(Debug, thiserror::Error)]
pub(crate) enum Error {
    #[error("Invalid packet type: {0}")]
    InvalidPacketType(u8),
}

pub enum Protocol {
    V311,
    V5,
}

/// 服务质量
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd)]
#[allow(clippy::enum_variant_names)]
pub enum QoS {
    AtMostOnce = 1,
    AtLeastOnce,
    ExactlyOnce,
}
