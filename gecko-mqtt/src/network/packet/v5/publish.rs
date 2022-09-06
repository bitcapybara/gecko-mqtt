use bytes::{Buf, BufMut, Bytes, BytesMut};

use crate::network::packet::{self, Error, FixedHeader, QoS};

use super::PropertyType;

#[derive(Debug)]
pub struct Publish {
    pub dup: bool,
    pub qos: QoS,
    pub retain: bool,
    pub topic: String,
    pub packet_id: u16,
    pub properties: Option<PublishProperties>,
    pub payload: Bytes,
}

impl Publish {
    fn len(&self) -> usize {
        let mut len = 2 + self.topic.len();
        if self.qos != QoS::AtMostOnce && self.packet_id != 0 {
            len += 2;
        }

        match &self.properties {
            Some(properties) => {
                let properties_len = properties.len();
                let properties_len_len = super::len_len(properties_len);
                len += properties_len_len + properties_len;
            }
            None => len += 1,
        }

        len
    }

    pub fn read(fixed_header: FixedHeader, mut stream: Bytes) -> Result<Self, Error> {
        let qos = ((fixed_header.byte1 & 0b0110) >> 1).try_into()?;
        let dup = (fixed_header.byte1 & 0b1000) != 0;
        let retain = (fixed_header.byte1 & 0b0001) != 0;

        let topic = packet::read_string(&mut stream)?;

        let packet_id = match qos {
            QoS::AtMostOnce => 0,
            QoS::AtLeastOnce | QoS::ExactlyOnce => packet::read_u16(&mut stream)?,
        };

        if qos != QoS::AtMostOnce && packet_id == 0 {
            return Err(Error::MissPacketId);
        }

        Ok(Self {
            dup,
            qos,
            retain,
            topic,
            packet_id,
            properties: PublishProperties::read(&mut stream)?,
            payload: stream,
        })
    }

    pub fn write(&self, stream: &mut BytesMut) -> Result<(), Error> {
        let dup = self.dup as u8;
        let qos = self.qos as u8;
        let retain = self.retain as u8;
        stream.put_u8(0b0011_0000 | retain | qos << 1 | dup << 3);

        packet::write_remaining_length(stream, self.len())?;
        packet::write_string(stream, &self.topic);

        if self.qos != QoS::AtMostOnce {
            let packet_id = self.packet_id;
            if packet_id == 0 {
                return Err(Error::MissPacketId);
            }

            stream.put_u16(packet_id);
        }

        match &self.properties {
            Some(properties) => {
                properties.write(stream)?;
            }
            None => {
                packet::write_remaining_length(stream, 0)?;
            }
        }

        stream.extend_from_slice(&self.payload);

        Ok(())
    }
}

#[derive(Debug)]
pub struct PublishProperties {
    pub payload_format_indicator: Option<u8>,
    pub message_expiry_interval: Option<u32>,
    pub topic_alias: Option<u16>,
    pub response_topic: Option<String>,
    pub correlation_data: Option<Bytes>,
    pub user_properties: Vec<(String, String)>,
    pub subscription_identifiers: Vec<usize>,
    pub content_type: Option<String>,
}

impl PublishProperties {
    fn len(&self) -> usize {
        let mut len = 0;

        if self.payload_format_indicator.is_some() {
            len += 1 + 1;
        }

        if self.message_expiry_interval.is_some() {
            len += 1 + 4;
        }

        if self.topic_alias.is_some() {
            len += 1 + 2;
        }

        if let Some(topic) = &self.response_topic {
            len += 1 + 2 + topic.len();
        }

        if let Some(data) = &self.correlation_data {
            len += 1 + 2 + data.len()
        }

        for (key, value) in &self.user_properties {
            len += 1 + 2 + key.len() + 2 + value.len();
        }

        for id in &self.subscription_identifiers {
            len += 1 + super::len_len(*id);
        }

        if let Some(typ) = &self.content_type {
            len += 1 + 2 + typ.len();
        }

        len
    }

    fn read(stream: &mut Bytes) -> Result<Option<Self>, Error> {
        let mut payload_format_indicator = None;
        let mut message_expiry_interval = None;
        let mut topic_alias = None;
        let mut response_topic = None;
        let mut correlation_data = None;
        let mut user_properties = Vec::new();
        let mut subscription_identifiers = Vec::new();
        let mut content_type = None;

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
                PropertyType::PayloadFormatIndicator => {
                    payload_format_indicator = Some(packet::read_u8(stream)?);
                    cursor += 1;
                }
                PropertyType::MessageExpiryInterval => {
                    message_expiry_interval = Some(packet::read_u32(stream)?);
                    cursor += 4;
                }
                PropertyType::TopicAlias => {
                    topic_alias = Some(packet::read_u16(stream)?);
                    cursor += 2;
                }
                PropertyType::ResponseTopic => {
                    let topic = packet::read_string(stream)?;
                    cursor += 2 + topic.len();
                    response_topic = Some(topic);
                }
                PropertyType::CorrelationData => {
                    let data = packet::read_bytes(stream)?;
                    cursor += 2 + data.len();
                    correlation_data = Some(data);
                }
                PropertyType::UserProperty => {
                    let key = packet::read_string(stream)?;
                    let value = packet::read_string(stream)?;
                    cursor += 2 + key.len() + 2 + value.len();
                    user_properties.push((key, value));
                }
                PropertyType::SubscriptionIdentifier => {
                    let (id_len, id) = packet::length(stream.iter())?;
                    cursor += 1 + id_len;
                    stream.advance(id_len);
                    subscription_identifiers.push(id);
                }
                PropertyType::ContentType => {
                    let typ = packet::read_string(stream)?;
                    cursor += 2 + typ.len();
                    content_type = Some(typ);
                }
                _ => return Err(Error::InvalidPacketType(prop)),
            }
        }

        Ok(Some(Self {
            payload_format_indicator,
            message_expiry_interval,
            topic_alias,
            response_topic,
            correlation_data,
            user_properties,
            subscription_identifiers,
            content_type,
        }))
    }

    fn write(&self, stream: &mut BytesMut) -> Result<(), Error> {
        packet::write_remaining_length(stream, self.len())?;

        if let Some(payload_format_indicator) = self.payload_format_indicator {
            stream.put_u8(PropertyType::PayloadFormatIndicator as u8);
            stream.put_u8(payload_format_indicator)
        }

        if let Some(message_expiry_interval) = self.message_expiry_interval {
            stream.put_u8(PropertyType::MessageExpiryInterval as u8);
            stream.put_u32(message_expiry_interval);
        }

        if let Some(topic_alias) = self.topic_alias {
            stream.put_u8(PropertyType::TopicAlias as u8);
            stream.put_u16(topic_alias);
        }

        if let Some(topic) = &self.response_topic {
            stream.put_u8(PropertyType::ResponseTopic as u8);
            packet::write_string(stream, topic);
        }

        if let Some(data) = &self.correlation_data {
            stream.put_u8(PropertyType::CorrelationData as u8);
            packet::write_bytes(stream, data);
        }

        for (key, value) in &self.user_properties {
            stream.put_u8(PropertyType::UserProperty as u8);
            packet::write_string(stream, key);
            packet::write_string(stream, value);
        }

        for id in &self.subscription_identifiers {
            stream.put_u8(PropertyType::SubscriptionIdentifier as u8);
            packet::write_remaining_length(stream, *id)?;
        }

        if let Some(typ) = &self.content_type {
            stream.put_u8(PropertyType::ContentType as u8);
            packet::write_string(stream, typ);
        }

        Ok(())
    }
}
