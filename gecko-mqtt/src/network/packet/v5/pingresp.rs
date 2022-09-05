use bytes::{BufMut, BytesMut};

use crate::network::packet::Error;

pub struct PingResp;

impl PingResp {
    pub fn write(&self, stream: &mut BytesMut) -> Result<(), Error> {
        stream.put_slice(&[0xD0, 0x00]);
        Ok(())
    }
}
