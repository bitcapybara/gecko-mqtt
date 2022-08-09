use bytes::{BufMut, BytesMut};
use tokio::net::TcpStream;

use crate::{error::Result, packet::Packet};

/// 设备与服务器之间的连接
/// 单纯的 tcp 读写管理
/// 以 packet 为单位读写
pub(crate) struct Connection {
    /// tcp 连接
    stream: TcpStream,
    /// 读缓冲区
    /// 使用缓冲区而非按照字节 从 socket 读取数据
    read: BytesMut,
    /// 写缓冲区
    /// 先写入缓冲区再刷入 socket 而非按字节向 socket 写入数据
    write: BytesMut,
    /// 上一次被批量读取出的 packet
    packets: Option<Vec<Packet>>,
}

impl Connection {
    /// 等待从 socket 读出至少所需长度的数据，放入缓冲区
    /// 如果读不到指定长度的数据，返回错误
    async fn read_bytes(_required: usize) -> Result<()> {
        // AsyncReadExt socket.read()
        // let mut total_read = 0;
        // loop {
        //     let read = self.socket.read_buf(&mut self.read).await?;
        //     if 0 == read {
        //         return if self.read.is_empty() {
        //             Err(io::Error::new(
        //                 ErrorKind::ConnectionAborted,
        //                 "connection closed by peer",
        //             ))
        //         } else {
        //             Err(io::Error::new(
        //                 ErrorKind::ConnectionReset,
        //                 "connection reset by peer",
        //             ))
        //         };
        //     }

        //     total_read += read;
        //     if total_read >= required {
        //         return Ok(total_read);
        //     }
        // }
        todo!()
    }

    /// 只从缓冲区读取指定长度的数据
    /// 如果缓冲区数据不足，返回 Insufficient 错误
    async fn read_u8(&self) -> Result<u8> {
        // 先从缓冲区读取，缓冲区不够，再从 socket 读取
        // loop {
        //     if self.read.len() >= 1 {
        //         // return
        //         todo!()
        //     }
        //     self.stream.read(&mut self.read).await?;
        // }
        todo!()
    }

    /// 把数据写入缓冲区
    async fn write_u8(&mut self, data: u8) -> Result<()> {
        self.write.reserve(1);
        self.write.put_u8(data);
        todo!()
    }

    /// 协议层处理完一个或多个请求后，主动调用此方法
    async fn flush(&self) -> Result<()> {
        todo!()
    }

    /// 从已读取的缓冲区中获取 packet 存入列表
    pub(crate) async fn read_packets(&mut self) -> Result<Vec<Packet>> {
        todo!()
    }
}
