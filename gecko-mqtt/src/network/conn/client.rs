use std::mem;

use bytes::BytesMut;
use packet::v4::ConnAck;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
    time,
};

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
    packets: Vec<Packet>,
}

impl ClientConnection {
    pub(crate) fn new(stream: TcpStream) -> Self {
        Self {
            stream,
            read: BytesMut::new(),
            write: BytesMut::new(),
            packets: Vec::with_capacity(10),
        }
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
    pub(crate) async fn read_more(&mut self, timeout: time::Duration) -> Result<(), Error> {
        loop {
            // 等待 keepalive 时间内至少有完整的包进来
            // 超时直接返回错误
            let timeout = time::timeout(timeout, self.read_packet()).await?;

            // 捕获 packet 读取错误
            match timeout {
                Ok(packet) => self.packets.push(packet),
                Err(Error::Packet(packet::Error::InsufficientBytes(_)))
                    if !self.packets.is_empty() =>
                {
                    return Ok(())
                }
                Err(Error::Packet(packet::Error::InsufficientBytes(required))) => {
                    self.read_bytes(required).await?
                }
                Err(e) => return Err(e),
            }
        }
    }

    pub(crate) fn packets(&mut self) -> Vec<Packet> {
        mem::replace(&mut self.packets, Vec::with_capacity(10))
    }

    /// 等待从 socket 读出至少所需长度的数据，放入缓冲区
    /// 如果读不到指定长度的数据，返回错误
    async fn read_bytes(&mut self, required: usize) -> Result<(), Error> {
        // AsyncReadExt socket.read()
        let mut total_read = 0;
        loop {
            let read = self.stream.read_buf(&mut self.read).await?;
            if 0 == read {
                todo!()
            }

            total_read += read;
            if total_read >= required {
                return Ok(());
            }
        }
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