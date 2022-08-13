use bytes::{BytesMut, BufMut};

use crate::network::packet::Error;

pub struct PingResp;

impl PingResp {
    pub fn write(&self, payload: &mut BytesMut) -> Result<(), Error> {
        payload.put_slice(&[0xD0, 0x00]);
        Ok(())
    }
}
