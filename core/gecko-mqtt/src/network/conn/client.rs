use bytes::{BufMut, BytesMut};
use packet::v4::ConnAck;
use tokio::{io::AsyncWriteExt, net::TcpStream};

use crate::network::{
    packet::{self, v4::Packet},
    v4::Connect,
};

use super::Error;

/// 设备或对等节点与服务器之间的连接
/// 单纯的 tcp 读写管理
/// 以 packet 为单位读写
pub(crate) struct ClientConnection {
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

impl ClientConnection {
    pub(crate) fn new(stream: TcpStream) -> Self {
        Self {
            stream,
            read: BytesMut::new(),
            write: BytesMut::new(),
            packets: None,
        }
    }

    /// 从已读取的缓冲区中获取 packet 存入列表
    pub(crate) async fn read_packets(&mut self) -> Result<Vec<Packet>, Error> {
        todo!()
    }

    /// 读取一个 packet
    async fn read_packet(&mut self) -> Result<Packet, Error> {
        loop {
            let required = match packet::v4::Packet::read_from(&mut self.read) {
                Ok(packet) => return Ok(packet),
                Err(packet::Error::InsufficientBytes(n)) => n,
                Err(e) => return Err(Error::Packet(e)),
            };

            // 数据不足，读取更多数据
            self.read_bytes(required).await?;
        }
    }

    pub(crate) async fn read_connect(&mut self) -> Result<Connect, Error> {
        let packet = self.read_packet().await?;

        match packet {
            Packet::Connect(connect) => Ok(connect),
            _ => Err(Error::FirstPacketNotConnect),
        }
    }

    pub(crate) async fn write_connack(&mut self, connack: ConnAck) -> Result<(), Error> {
        connack.write(&mut self.write)?;
        self.flush().await?;
        Ok(())
    }

    pub(crate) async fn write_packet(&mut self, packet: Packet) -> Result<(), Error> {
        Ok(packet.write(&mut self.write)?)
    }

    /// 从 socket 读取更多数据
    async fn read(&mut self) -> Result<(), Error> {
        todo!()
    }

    /// 等待从 socket 读出至少所需长度的数据，放入缓冲区
    /// 如果读不到指定长度的数据，返回错误
    async fn read_bytes(&mut self, _required: usize) -> Result<(), Error> {
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
    async fn read_u8(&self) -> Result<u8, Error> {
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
    async fn write_u8(&mut self, data: u8) -> Result<(), Error> {
        self.write.reserve(1);
        self.write.put_u8(data);
        todo!()
    }

    /// 协议层处理完一个或多个请求后，主动调用此方法
    async fn flush(&mut self) -> Result<(), Error> {
        if self.write.is_empty() {
            return Ok(());
        }

        self.stream.write_all(&self.write).await?;
        self.write.clear();
        Ok(())
    }
}
