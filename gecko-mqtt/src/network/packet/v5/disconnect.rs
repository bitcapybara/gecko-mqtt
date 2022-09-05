use bytes::{Buf, BufMut, Bytes, BytesMut};

use crate::network::packet::{self, Error, FixedHeader};

use super::PropertyType;

#[derive(Debug)]
pub struct Disconnect {
    /// 断开连接原因码
    pub reason_code: DisconnectReasonCode,
    /// 断开连接属性
    pub properties: Option<DisconnectProperties>,
}

impl Disconnect {
    pub fn len(&self) -> usize {
        if self.reason_code == DisconnectReasonCode::NormalDisconnection
            && self.properties.is_none()
        {
            return 2;
        }

        let mut length = 0;
        match &self.properties {
            Some(properties) => {
                length += 1;
                let properties_len = properties.len();
                let properties_len_len = super::len_len(properties_len);
                length += properties_len_len + properties_len;
            }
            None => length += 1,
        }

        length
    }

    pub fn read(fixed_header: FixedHeader, mut stream: Bytes) -> Result<Self, Error> {
        if fixed_header.byte1 & 0b0000_1111 != 0x00 {
            return Err(Error::MalformedPacket);
        }

        if fixed_header.remaining_len == 0 {
            return Ok(Self {
                reason_code: DisconnectReasonCode::NormalDisconnection,
                properties: None,
            });
        }

        Ok(Self {
            reason_code: packet::read_u8(&mut stream)?.try_into()?,
            properties: DisconnectProperties::read(&mut stream)?,
        })
    }

    pub fn write(&self, stream: &mut BytesMut) -> Result<(), Error> {
        stream.put_u8(0xE0);

        let len = self.len();
        if len == 2 {
            stream.put_u8(0x00);
            return Ok(());
        }
        packet::write_remaining_length(stream, len)?;
        stream.put_u8(self.reason_code as u8);

        match &self.properties {
            Some(properties) => {
                properties.write(stream)?;
            }
            None => {
                packet::write_remaining_length(stream, 0)?;
            }
        }
        Ok(())
    }
}

#[derive(Debug)]
pub struct DisconnectProperties {
    /// 会话过期时间（秒）
    pub session_expiry_interval: Option<u32>,
    /// 人类可读的断开连接原因
    pub reason_string: Option<String>,
    /// 用户属性列表
    pub user_properties: Vec<(String, String)>,
    /// 服务器节点标识符
    pub server_reference: Option<String>,
}

impl DisconnectProperties {
    fn len(&self) -> usize {
        let mut len = 0;

        if self.session_expiry_interval.is_some() {
            len += 1 + 4;
        }

        if let Some(reason) = &self.reason_string {
            len += 1 + 2 + reason.len();
        }

        for (key, value) in &self.user_properties {
            len += 1 + 2 + key.len() + 2 + value.len();
        }

        if let Some(server_reference) = &self.server_reference {
            len += 1 + 2 + server_reference.len();
        }

        len
    }

    fn read(stream: &mut Bytes) -> Result<Option<Self>, Error> {
        let (properties_len_len, properties_len) = packet::length(stream.iter())?;
        stream.advance(properties_len_len);
        if properties_len == 0 {
            return Ok(None);
        }

        let mut session_expiry_interval = None;
        let mut reason_string = None;
        let mut user_properties = Vec::new();
        let mut server_reference = None;

        let mut cursor = 0;
        while cursor < properties_len {
            let prop = packet::read_u8(stream)?;
            cursor += 1;

            match prop.try_into()? {
                PropertyType::SessionExpiryInterval => {
                    session_expiry_interval = Some(packet::read_u32(stream)?);
                    cursor += 4;
                }
                PropertyType::ReasonString => {
                    let reason = packet::read_string(stream)?;
                    cursor += 2 + reason.len();
                    reason_string = Some(reason);
                }
                PropertyType::UserProperty => {
                    let key = packet::read_string(stream)?;
                    let value = packet::read_string(stream)?;
                    cursor += 2 + key.len() + 2 + value.len();
                    user_properties.push((key, value));
                }
                PropertyType::ServerReference => {
                    let reference = packet::read_string(stream)?;
                    cursor += 2 + reference.len();
                    server_reference = Some(reference);
                }
                _ => return Err(Error::InvalidPacketType(prop)),
            }
        }

        Ok(Some(Self {
            session_expiry_interval,
            reason_string,
            user_properties,
            server_reference,
        }))
    }

    fn write(&self, stream: &mut BytesMut) -> Result<(), Error> {
        packet::write_remaining_length(stream, self.len())?;

        if let Some(session_expiry_interval) = self.session_expiry_interval {
            stream.put_u8(PropertyType::SessionExpiryInterval as u8);
            stream.put_u32(session_expiry_interval);
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

        if let Some(reference) = &self.server_reference {
            stream.put_u8(PropertyType::ServerReference as u8);
            packet::write_string(stream, reference);
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum DisconnectReasonCode {
    /// 正常关闭，不发送遗嘱消息
    NormalDisconnection = 0x00,
    /// 客户端期望关闭但是要求服务端发送遗嘱消息（客户端使用）
    DisconnectWithWillMessage = 0x04,
    /// 连接已关闭，但发送方不希望透露原因，或者其他原因代码均不适用。
    UnspecifiedError = 0x80,
    /// 接收到的数据包不符合本规范。
    MalformedPacket = 0x81,
    /// 收到了未期望或无序的数据包。
    ProtocolError = 0x82,
    /// 接收到的数据包是有效的，但不能被这个实现处理。
    ImplementationSpecificError = 0x83,
    /// 该请求未经授权。
    NotAuthorized = 0x87,
    /// 服务器正忙，无法继续处理来自该客户端的请求。
    /// TODO 比如，router 队列已满的情况
    ServerBusy = 0x89,
    /// 服务器正在关闭。
    ServerShuttingDown = 0x8B,
    /// 连接被关闭，因为在 Keepalive 时间的 1.5 倍内没有收到任何数据包。
    KeepAliveTimeout = 0x8D,
    /// 另一个使用相同 ClientID 的连接已连接，导致此连接被关闭。
    SessionTakenOver = 0x8E,
    /// 主题过滤器格式正确，但不被此服务器接受。
    TopicFilterInvalid = 0x8F,
    /// 主题名称格式正确，但此客户端或服务器不接受。
    TopicNameInvalid = 0x90,
    /// 客户端或服务器已收到超过其尚未发送 PUBACK 或 PUBCOMP 的 Receive Maximum 发布。
    ReceiveMaximumExceeded = 0x93,
    /// 客户端或服务器已收到包含主题别名的 PUBLISH 数据包，该主题别名大于它在 CONNECT 或 CONNACK 数据包中发送的最大主题别名。
    TopicAliasInvalid = 0x94,
    /// 数据包大小大于此客户端或服务器的最大数据包大小。
    /// TODO 配置
    PacketTooLarge = 0x95,
    /// 接收数据速率太高。
    /// TODO 配置
    MessageRateTooHigh = 0x96,
    /// 已超出实施或管理员规定的限制。
    QuotaExceeded = 0x97,
    /// 由于管理员操作，连接已关闭。
    /// TODO 接口强制关闭
    AdministrativeAction = 0x98,
    /// 有效负载格式与有效负载格式指示符指定的格式不匹配。
    PayloadFormatInvalid = 0x99,
    /// 服务器不支持保留消息。
    RetainNotSupported = 0x9A,
    /// 客户端 publish 指定的 QoS 大于 CONNACK 中最大 QoS 中指定的 QoS。
    QoSNotSupported = 0x9B,
    /// 客户端应临时更改其服务器。
    UseAnotherServer = 0x9C,
    /// 服务器已移动，客户端应永久更改其服务器位置。
    ServerMoved = 0x9D,
    /// 服务器不支持共享订阅。
    SharedSubscriptionNotSupported = 0x9E,
    /// 此连接已关闭，因为连接速率太高。
    ConnectionRateExceeded = 0x9F,
    /// 已超过为此连接授权的最大连接时间。
    MaximumConnectTime = 0xA0,
    /// 服务器不支持订阅标识符； 不接受订阅。
    SubscriptionIdentifiersNotSupported = 0xA1,
    /// 服务器不支持通配符订阅； 不接受订阅。
    WildcardSubscriptionsNotSupported = 0xA2,
}

impl TryFrom<u8> for DisconnectReasonCode {
    type Error = Error;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        let rc = match value {
            0x00 => Self::NormalDisconnection,
            0x04 => Self::DisconnectWithWillMessage,
            0x80 => Self::UnspecifiedError,
            0x81 => Self::MalformedPacket,
            0x82 => Self::ProtocolError,
            0x83 => Self::ImplementationSpecificError,
            0x87 => Self::NotAuthorized,
            0x89 => Self::ServerBusy,
            0x8B => Self::ServerShuttingDown,
            0x8D => Self::KeepAliveTimeout,
            0x8E => Self::SessionTakenOver,
            0x8F => Self::TopicFilterInvalid,
            0x90 => Self::TopicNameInvalid,
            0x93 => Self::ReceiveMaximumExceeded,
            0x94 => Self::TopicAliasInvalid,
            0x95 => Self::PacketTooLarge,
            0x96 => Self::MessageRateTooHigh,
            0x97 => Self::QuotaExceeded,
            0x98 => Self::AdministrativeAction,
            0x99 => Self::PayloadFormatInvalid,
            0x9A => Self::RetainNotSupported,
            0x9B => Self::QoSNotSupported,
            0x9C => Self::UseAnotherServer,
            0x9D => Self::ServerMoved,
            0x9E => Self::SharedSubscriptionNotSupported,
            0x9F => Self::ConnectionRateExceeded,
            0xA0 => Self::MaximumConnectTime,
            0xA1 => Self::SubscriptionIdentifiersNotSupported,
            0xA2 => Self::WildcardSubscriptionsNotSupported,
            other => return Err(super::Error::InvalidReasonCode(other))?,
        };

        Ok(rc)
    }
}
