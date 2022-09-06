use bytes::{Buf, Bytes};

use crate::network::packet::{self, Error, QoS};

use super::PropertyType;

#[derive(Debug)]
pub struct Subscribe {
    pub packet_id: u16,
    pub filters: Vec<SubscribeFilter>,
    pub properties: Option<SubscribeProperties>,
}

impl Subscribe {
    pub fn read(mut stream: Bytes) -> Result<Self, Error> {
        let packet_id = packet::read_u16(&mut stream)?;
        let properties = SubscribeProperties::read(&mut stream)?;

        let mut filters = Vec::new();
        while stream.has_remaining() {
            let filter = packet::read_string(&mut stream)?;
            let flags = packet::read_u8(&mut stream)?;
            let qos = flags & 0b0000_0011;

            let nolocal = flags >> 2 & 0b0000_0001;
            let nolocal = nolocal != 0;

            let preserve_retain = flags >> 3 & 0b0000_0001;
            let preserve_retain = preserve_retain != 0;

            let retain_forward_rule = (flags >> 4) & 0b0000_0011;
            let retain_forward_rule = match retain_forward_rule {
                0 => RetainForwardRule::OnEverySubscribe,
                1 => RetainForwardRule::OnNewSubscribe,
                2 => RetainForwardRule::Never,
                r => return Err(super::Error::InvalidRetainForwardRule(r))?,
            };

            filters.push(SubscribeFilter {
                filter,
                qos: qos.try_into()?,
                nolocal,
                preserve_retain,
                retain_forward_rule,
            })
        }

        if filters.is_empty() {
            return Err(super::Error::EmptySubscription)?;
        }

        Ok(Self {
            packet_id,
            filters,
            properties,
        })
    }
}

#[derive(Debug)]
pub struct SubscribeFilter {
    pub filter: String,
    pub qos: QoS,
    pub nolocal: bool,
    pub preserve_retain: bool,
    pub retain_forward_rule: RetainForwardRule,
}

#[derive(Debug)]
pub enum RetainForwardRule {
    OnEverySubscribe,
    OnNewSubscribe,
    Never,
}

#[derive(Debug)]
pub struct SubscribeProperties {
    pub id: Option<usize>,
    pub user_properties: Vec<(String, String)>,
}

impl SubscribeProperties {
    pub fn read(stream: &mut Bytes) -> Result<Option<Self>, Error> {
        let mut id = None;
        let mut user_properties = Vec::new();

        let (properties_len_len, properties_len) = packet::length(stream.iter())?;
        stream.advance(properties_len_len);
        if properties_len == 0 {
            return Ok(None);
        }

        let mut cursor = 0;
        while cursor < properties_len {
            let prop = packet::read_u8(stream)?;
            cursor += 1;

            match prop.try_into()? {
                PropertyType::SubscriptionIdentifier => {
                    let (id_len, sub_id) = packet::length(stream.iter())?;
                    cursor += 1 + id_len;
                    stream.advance(id_len);
                    id = Some(sub_id);
                }
                PropertyType::UserProperty => {
                    let key = packet::read_string(stream)?;
                    let value = packet::read_string(stream)?;
                    cursor += 2 + key.len() + 2 + value.len();
                    user_properties.push((key, value));
                }
                _ => return Err(Error::InvalidPacketType(prop)),
            }
        }

        Ok(Some(Self {
            id,
            user_properties,
        }))
    }
}
