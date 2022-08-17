use bytes::Bytes;

use crate::network::packet::{self, Error, Protocol, QoS};

#[derive(Debug)]
pub struct Connect {
    /// 协议版本
    pub protocol: Protocol,
    /// keepalive
    pub keep_alive: u16,
    /// 客户端id
    pub client_id: String,
    /// 是否开启新会话
    pub clean_session: bool,
    /// 遗嘱消息
    pub last_will: Option<LastWill>,
    /// 登录凭证
    pub login: Option<Login>,
}

impl Connect {
    pub(crate) fn read_from(mut stream: Bytes) -> Result<Self, Error> {
        // 可变报头
        let protocol_name = packet::read_string(&mut stream)?;
        let protocol_level = packet::read_u8(&mut stream)?;
        if protocol_name != "MQTT" {
            return Err(Error::InvalidProtocol);
        }
        let protocol = match protocol_level {
            4 => Protocol::V4,
            5 => Protocol::V5,
            num => return Err(Error::InvalidProtocolLevel(num)),
        };

        let connect_flags = packet::read_u8(&mut stream)?;
        let clean_session = (connect_flags & 0b10) != 0;
        let keep_alive = packet::read_u16(&mut stream)?;

        let client_id = packet::read_string(&mut stream)?;
        let last_will = LastWill::read(connect_flags, &mut stream)?;
        let login = Login::read(connect_flags, &mut stream)?;

        Ok(Connect {
            protocol,
            keep_alive,
            client_id,
            clean_session,
            last_will,
            login,
        })
    }
}

/// 遗嘱设置
#[derive(Debug)]
pub struct LastWill {
    /// 遗嘱发送的目标主题
    pub topic: String,
    // 遗嘱消息
    pub message: Bytes,
    /// 服务质量
    pub qos: QoS,
    /// 消息保留
    pub retain: bool,
}

impl LastWill {
    fn read(connect_flags: u8, stream: &mut Bytes) -> Result<Option<LastWill>, Error> {
        let last_will = match connect_flags & 0b100 {
            0 if (connect_flags & 0b0011_1000) != 0 => {
                return Err(Error::IncorrectPacketFormat);
            }
            0 => None,
            _ => Some(LastWill {
                topic: packet::read_string(stream)?,
                message: packet::read_bytes(stream)?,
                qos: QoS::try_from((connect_flags & 0b11000) >> 3)?,
                retain: (connect_flags & 0b0010_0000) != 0,
            }),
        };

        Ok(last_will)
    }
}

/// 登录凭证
#[derive(Debug, Clone)]
pub struct Login {
    /// 用户名
    pub username: String,
    /// 密码
    pub password: String,
}

impl Login {
    fn read(connect_flags: u8, stream: &mut Bytes) -> Result<Option<Login>, Error> {
        let username = match connect_flags & 0b1000_0000 {
            0 => None,
            _ => Some(packet::read_string(stream)?),
        };

        let password = match connect_flags & 0b0100_0000 {
            0 => None,
            _ => Some(packet::read_string(stream)?),
        };

        let login = match (&username, &password) {
            (None, None) => None,
            _ => Some(Login {
                username: username.unwrap_or_default(),
                password: password.unwrap_or_default(),
            }),
        };

        Ok(login)
    }
}
