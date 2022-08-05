use bytes::BytesMut;
use tokio::net::TcpStream;

// 单纯的 tcp 读写管理
pub(crate) struct Connection {
    // tcp 连接
    stream: TcpStream,
    // 读缓冲区
    // 读取报文时，首先尝试从缓冲区读取，无法读取到时，再从 socket读取
    read: BytesMut,
    // 写缓冲区
    // 写入报文时，先写入缓冲区，后台循环线程定期写入 socket
    write: BytesMut
}