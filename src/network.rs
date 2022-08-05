use tokio::net::TcpStream;

use crate::{error::Result, connection::Connection};

// 网络层
pub(crate) struct Network {
    // tcp 连接
    conn: Connection
}

impl Network {
    
    fn new(_stream: TcpStream) -> Result<Self> {
        // TODO 读取 connect 消息，校验
        todo!()
    }

    async fn start() -> Result<()> {
        // 循环读取 tcp 报文，交给 router 处理
        todo!()
    }
}
