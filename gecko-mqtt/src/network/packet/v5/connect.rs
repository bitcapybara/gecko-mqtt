use bytes::{Buf, Bytes};

use crate::network::packet::{self, Error, Protocol, QoS};

use super::PropertyType;

#[derive(Debug)]
pub struct Connect {
    /// mqtt 协议版本
    pub protocol: Protocol,
    /// keepalive 时长
    pub keepalive: u16,
    /// 客户端id
    pub client_id: String,
    /// 是否清除会话
    pub clean_start: bool,
    /// 遗嘱消息
    pub last_will: Option<LastWill>,
    /// 登录凭证
    pub login: Login,
    /// 属性
    pub properties: Option<ConnectProperties>,
}

impl Connect {
    pub fn read(mut stream: Bytes) -> Result<Self, Error> {
        let protocol_name = packet::read_string(&mut stream)?;
        if protocol_name != "MQTT" {
            return Err(Error::InvalidProtocol);
        }
        let protocol_level = packet::read_u8(&mut stream)?;

        let protocol = match protocol_level {
            4 => Protocol::V4,
            5 => Protocol::V5,
            num => return Err(Error::InvalidProtocolLevel(num)),
        };

        let connect_flags = packet::read_u8(&mut stream)?;
        let clean_start = (connect_flags & 0b10) != 0;
        let keepalive = packet::read_u16(&mut stream)?;

        let properties = match protocol {
            Protocol::V4 => None,
            Protocol::V5 => ConnectProperties::read(&mut stream)?,
        };

        let client_id = packet::read_string(&mut stream)?;
        let last_will = LastWill::read(connect_flags, &mut stream)?;
        let login = Login::read(connect_flags, &mut stream)?;

        Ok(Self {
            protocol,
            keepalive,
            client_id,
            clean_start,
            last_will,
            login,
            properties,
        })
    }
}

#[derive(Debug)]
pub struct LastWill {
    pub topic: String,
    pub message: Bytes,
    pub qos: QoS,
    pub retain: bool,
    pub properties: Option<WillProperties>,
}

impl LastWill {
    fn read(connect_flags: u8, stream: &mut Bytes) -> Result<Option<Self>, Error> {
        let last_will = match connect_flags & 0b100 {
            0 if (connect_flags & 0b0011_1000) != 0 => return Err(Error::MalformedPacket),
            0 => None,
            _ => {
                let properties = WillProperties::read(stream)?;
                let topic = packet::read_string(stream)?;
                let message = packet::read_bytes(stream)?;
                let qos = ((connect_flags & 0b11000) >> 3).try_into()?;
                let retain = (connect_flags & 0b0010_0000) != 0;

                Some(Self {
                    topic,
                    message,
                    qos,
                    retain,
                    properties,
                })
            }
        };

        Ok(last_will)
    }
}

#[derive(Debug)]
pub struct WillProperties {
    pub delay_interval: Option<u32>,
    pub payload_format_indicator: Option<u8>,
    pub message_expiry_interval: Option<u32>,
    pub content_type: Option<String>,
    pub response_topic: Option<String>,
    pub correlation_data: Option<Bytes>,
    pub user_properties: Vec<(String, String)>,
}

impl WillProperties {
    fn read(stream: &mut Bytes) -> Result<Option<WillProperties>, Error> {
        let mut delay_interval = None;
        let mut payload_format_indicator = None;
        let mut message_expiry_interval = None;
        let mut content_type = None;
        let mut response_topic = None;
        let mut correlation_data = None;
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
                PropertyType::WillDelayInterval => {
                    delay_interval = Some(packet::read_u32(stream)?);
                    cursor += 4;
                }
                PropertyType::PayloadFormatIndicator => {
                    payload_format_indicator = Some(packet::read_u8(stream)?);
                    cursor += 1;
                }
                PropertyType::MessageExpiryInterval => {
                    message_expiry_interval = Some(packet::read_u32(stream)?);
                    cursor += 4;
                }
                PropertyType::ContentType => {
                    let typ = packet::read_string(stream)?;
                    cursor += 2 + typ.len();
                    content_type = Some(typ);
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
                _ => return Err(super::Error::UnexpectedPropertyType(prop))?,
            }
        }
        Ok(Some(Self {
            delay_interval,
            payload_format_indicator,
            message_expiry_interval,
            content_type,
            response_topic,
            correlation_data,
            user_properties,
        }))
    }
}

#[derive(Debug)]
pub struct Login {
    pub username: Option<String>,
    pub password: Option<String>,
}

impl Login {
    fn read(connect_flags: u8, stream: &mut Bytes) -> Result<Self, Error> {
        let username = match connect_flags & 0b1000_0000 {
            0 => None,
            _ => Some(packet::read_string(stream)?),
        };

        let password = match connect_flags & 0b0100_0000 {
            0 => None,
            _ => Some(packet::read_string(stream)?),
        };

        Ok(Self { username, password })
    }
}

#[derive(Debug)]
pub struct ConnectProperties {
    pub session_expiry_interval: Option<u32>,
    pub receive_maximum: Option<u16>,
    pub max_packet_size: Option<u32>,
    pub topic_alias_max: Option<u16>,
    pub request_response_info: Option<u8>,
    pub request_problem_info: Option<u8>,
    pub user_properties: Vec<(String, String)>,
    pub authentication_method: Option<String>,
    pub authentication_data: Option<Bytes>,
}

impl ConnectProperties {
    fn read(stream: &mut Bytes) -> Result<Option<Self>, Error> {
        let mut session_expiry_interval = None;
        let mut receive_maximum = None;
        let mut max_packet_size = None;
        let mut topic_alias_max = None;
        let mut request_response_info = None;
        let mut request_problem_info = None;
        let mut user_properties = Vec::new();
        let mut authentication_method = None;
        let mut authentication_data = None;

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
                PropertyType::SessionExpiryInterval => {
                    session_expiry_interval = Some(packet::read_u32(stream)?);
                    cursor += 4;
                }
                PropertyType::ReceiveMaximum => {
                    receive_maximum = Some(packet::read_u16(stream)?);
                    cursor += 2;
                }
                PropertyType::MaximumPacketSize => {
                    max_packet_size = Some(packet::read_u32(stream)?);
                    cursor += 4;
                }
                PropertyType::TopicAliasMaximum => {
                    topic_alias_max = Some(packet::read_u16(stream)?);
                    cursor += 2;
                }
                PropertyType::RequestResponseInformation => {
                    request_response_info = Some(packet::read_u8(stream)?);
                    cursor += 1;
                }
                PropertyType::RequestProblemInformation => {
                    request_problem_info = Some(packet::read_u8(stream)?);
                    cursor += 1;
                }
                PropertyType::UserProperty => {
                    let key = packet::read_string(stream)?;
                    let value = packet::read_string(stream)?;
                    cursor += 2 + key.len() + 2 + value.len();
                    user_properties.push((key, value));
                }
                PropertyType::AuthenticationMethod => {
                    let method = packet::read_string(stream)?;
                    cursor += 2 + method.len();
                    authentication_method = Some(method);
                }
                PropertyType::AuthenticationData => {
                    let data = packet::read_bytes(stream)?;
                    cursor += 2 + data.len();
                    authentication_data = Some(data);
                }
                _ => return Err(super::Error::UnexpectedPropertyType(prop))?,
            }
        }

        Ok(Some(Self {
            session_expiry_interval,
            receive_maximum,
            max_packet_size,
            topic_alias_max,
            request_response_info,
            request_problem_info,
            user_properties,
            authentication_method,
            authentication_data,
        }))
    }
}
