/// 连接返回码
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

pub struct ConnAck {
    /// 用于标识在 Broker 上是否已存在该 Client的持久性会话
    pub session_present: bool,
    /// 连接返回码
    pub code: ConnectReturnCode,
}
