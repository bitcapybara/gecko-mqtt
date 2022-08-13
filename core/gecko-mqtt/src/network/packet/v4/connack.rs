use bytes::{BufMut, BytesMut};

use crate::network::packet::{write_remaining_length, Error};

/// 连接返回码
#[derive(Debug, Copy, Clone)]
#[repr(u8)]
pub enum ConnectReturnCode {
    /// 成功
    Success = 0,
    /// 服务器不支持协议版本
    RefusedProtocolVersion,
    /// 客户端id不合法，比如长度超过 23 个字符，包含了不允许的字符等
    BadClientId,
    /// 服务器不可用
    ServiceUnavailable,
    /// 错误的用户名或密码
    BadUserNamePassword,
    /// 未授权
    NotAuthorized,
}

#[derive(Debug)]
pub struct ConnAck {
    /// 用于标识在 Broker 上是否已存在该 Client的持久性会话
    pub session_present: bool,
    /// 连接返回码
    pub code: ConnectReturnCode,
}

impl ConnAck {
    pub fn new(code: ConnectReturnCode, session_present: bool) -> Self {
        ConnAck {
            session_present,
            code,
        }
    }

    /// 报文长度
    pub(crate) fn len(&self) -> usize {
        // sesssion present + code
        1 + 1
    }

    pub fn write(&self, stream: &mut BytesMut) -> Result<(), Error> {
        stream.put_u8(0x20);

        let len = self.len();
        write_remaining_length(stream, len)?;
        stream.put_u8(self.session_present as u8);
        stream.put_u8(self.code as u8);

        Ok(())
    }
}
