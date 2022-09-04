use bytes::{BufMut, Bytes, BytesMut};

use crate::network::packet::{self, Error};

use super::PropertyType;

#[derive(Debug)]
pub struct ConnAck {
    pub session_present: bool,
    pub code: ConnectReturnCode,
    pub properties: Option<ConnAckProperties>,
}

impl ConnAck {
    fn len(&self) -> usize {
        let mut len = 1 + 1;
        if let Some(properties) = &self.properties {
            let properties_len = properties.len();
            let properties_len_len = super::len_len(len);
            len += properties_len_len + properties_len;
        } else {
            len += 1;
        }

        len
    }

    pub fn write(&self, stream: &mut BytesMut) -> Result<(), Error> {
        stream.put_u8(0x20);

        packet::write_remaining_length(stream, self.len())?;
        stream.put_u8(self.session_present as u8);
        stream.put_u8(self.code as u8);

        if let Some(properties) = &self.properties {
            properties.write(stream)?;
        } else {
            packet::write_remaining_length(stream, 0)?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum ConnectReturnCode {
    Success = 0,
    UnspecifiedError = 128,
    MalformedPacket = 129,
    ProtocolError = 130,
    ImplementationSpecificError = 131,
    UnsupportedProtocolVersion = 132,
    ClientIdentifierNotValid = 133,
    BadUserNamePassword = 134,
    NotAuthorized = 135,
    ServerUnavailable = 136,
    ServerBusy = 137,
    Banned = 138,
    BadAuthenticationMethod = 140,
    TopicNameInvalid = 144,
    PacketTooLarge = 149,
    QuotaExceeded = 151,
    PayloadFormatInvalid = 153,
    RetainNotSupported = 154,
    QoSNotSupported = 155,
    UseAnotherServer = 156,
    ServerMoved = 157,
    ConnectionRateExceeded = 159,
}

#[derive(Debug)]
pub struct ConnAckProperties {
    pub session_expiry_interval: Option<u32>,
    pub receive_max: Option<u16>,
    pub max_qos: Option<u8>,
    pub retain_available: Option<u8>,
    pub max_packet_size: Option<u32>,
    pub assigned_client_identifier: Option<String>,
    pub topic_alias_max: Option<u16>,
    pub reason_string: Option<String>,
    pub user_properties: Vec<(String, String)>,
    pub wildcard_subscription_available: Option<u8>,
    pub subscription_identifiers_available: Option<u8>,
    pub shared_subscription_available: Option<u8>,
    pub server_keep_alive: Option<u16>,
    pub response_information: Option<String>,
    pub server_reference: Option<String>,
    pub authentication_method: Option<String>,
    pub authentication_data: Option<Bytes>,
}

impl ConnAckProperties {
    fn len(&self) -> usize {
        let mut len = 0;

        if self.session_expiry_interval.is_some() {
            len += 1 + 4;
        }

        if self.receive_max.is_some() {
            len += 1 + 2;
        }

        if self.max_qos.is_some() {
            len += 1 + 1;
        }

        if self.retain_available.is_some() {
            len += 1 + 1;
        }

        if self.max_packet_size.is_some() {
            len += 1 + 4;
        }

        if let Some(id) = &self.assigned_client_identifier {
            len += 1 + 2 + id.len();
        }

        if self.topic_alias_max.is_some() {
            len += 1 + 2;
        }

        if let Some(reason) = &self.reason_string {
            len += 1 + 2 + reason.len();
        }

        for (key, value) in &self.user_properties {
            len += 1 + 2 + key.len() + 2 + value.len();
        }

        if self.wildcard_subscription_available.is_some() {
            len += 1 + 1;
        }

        if self.subscription_identifiers_available.is_some() {
            len += 1 + 1;
        }

        if self.shared_subscription_available.is_some() {
            len += 1 + 1;
        }

        if self.server_keep_alive.is_some() {
            len += 1 + 1;
        }

        if let Some(info) = &self.response_information {
            len += 1 + 2 + info.len();
        }

        if let Some(reference) = &self.server_reference {
            len += 1 + 2 + reference.len();
        }

        if let Some(authentication_method) = &self.authentication_method {
            len += 1 + 2 + authentication_method.len();
        }

        if let Some(authentication_data) = &self.authentication_data {
            len += 1 + 2 + authentication_data.len();
        }

        len
    }

    fn write(&self, stream: &mut BytesMut) -> Result<(), Error> {
        packet::write_remaining_length(stream, self.len())?;

        if let Some(session_expiry_interval) = self.session_expiry_interval {
            stream.put_u8(PropertyType::SessionExpiryInterval as u8);
            stream.put_u32(session_expiry_interval);
        }

        if let Some(receive_maximum) = self.receive_max {
            stream.put_u8(PropertyType::ReceiveMaximum as u8);
            stream.put_u16(receive_maximum);
        }

        if let Some(qos) = self.max_qos {
            stream.put_u8(PropertyType::MaximumQos as u8);
            stream.put_u8(qos);
        }

        if let Some(retain_available) = self.retain_available {
            stream.put_u8(PropertyType::RetainAvailable as u8);
            stream.put_u8(retain_available);
        }

        if let Some(max_packet_size) = self.max_packet_size {
            stream.put_u8(PropertyType::MaximumPacketSize as u8);
            stream.put_u32(max_packet_size);
        }

        if let Some(id) = &self.assigned_client_identifier {
            stream.put_u8(PropertyType::AssignedClientIdentifier as u8);
            packet::write_string(stream, id);
        }

        if let Some(topic_alias_max) = self.topic_alias_max {
            stream.put_u8(PropertyType::TopicAliasMaximum as u8);
            stream.put_u16(topic_alias_max);
        }

        if let Some(reason) = &self.reason_string {
            stream.put_u8(PropertyType::ReasonString as u8);
            packet::write_string(stream, reason);
        }

        for (key, value) in &self.user_properties {
            stream.put_u8(PropertyType::UserProperty as u8);
            packet::write_string(stream, key);
            packet::write_string(stream, value);
        }

        if let Some(wildcard_subscription_avalable) = self.wildcard_subscription_available {
            stream.put_u8(PropertyType::WildcardSubscriptionAvailable as u8);
            stream.put_u8(wildcard_subscription_avalable);
        }

        if let Some(subscription_identifiers_available) = self.subscription_identifiers_available {
            stream.put_u8(PropertyType::SubscriptionIdentifierAvailable as u8);
            stream.put_u8(subscription_identifiers_available);
        }

        if let Some(shared_subscription_available) = self.shared_subscription_available {
            stream.put_u8(PropertyType::SharedSubscriptionAvailable as u8);
            stream.put_u8(shared_subscription_available);
        }

        if let Some(keep_alive) = self.server_keep_alive {
            stream.put_u8(PropertyType::ServerKeepAlive as u8);
            stream.put_u16(keep_alive);
        }

        if let Some(info) = &self.response_information {
            stream.put_u8(PropertyType::ResponseInformation as u8);
            packet::write_string(stream, info);
        }

        if let Some(reference) = &self.server_reference {
            stream.put_u8(PropertyType::ServerReference as u8);
            packet::write_string(stream, reference);
        }

        if let Some(authentication_method) = &self.authentication_method {
            stream.put_u8(PropertyType::AuthenticationMethod as u8);
            packet::write_string(stream, authentication_method);
        }

        if let Some(authentication_data) = &self.authentication_data {
            stream.put_u8(PropertyType::AuthenticationData as u8);
            packet::write_bytes(stream, authentication_data);
        }

        Ok(())
    }
}
