use bytes::{Buf, Bytes};

use crate::network::packet::{self, Error};

use super::PropertyType;

#[derive(Debug)]
pub struct Unsubscribe {
    pub packet_id: u16,
    pub filters: Vec<String>,
    pub properties: Option<UnsubscribeProperties>,
}

impl Unsubscribe {
    pub fn read(mut stream: Bytes) -> Result<Self, Error> {
        let packet_id = packet::read_u16(&mut stream)?;
        let properties = UnsubscribeProperties::read(&mut stream)?;

        let mut filters = Vec::new();
        while stream.has_remaining() {
            let filter = packet::read_string(&mut stream)?;
            filters.push(filter);
        }

        Ok(Self {
            packet_id,
            filters,
            properties,
        })
    }
}

#[derive(Debug)]
pub struct UnsubscribeProperties {
    pub user_properties: Vec<(String, String)>,
}

impl UnsubscribeProperties {
    fn read(stream: &mut Bytes) -> Result<Option<Self>, Error> {
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
                PropertyType::UserProperty => {
                    let key = packet::read_string(stream)?;
                    let value = packet::read_string(stream)?;
                    cursor += 2 + key.len() + 2 + value.len();
                    user_properties.push((key, value));
                }
                _ => return Err(Error::InvalidPacketType(prop)),
            }
        }

        Ok(Some(Self { user_properties }))
    }
}
