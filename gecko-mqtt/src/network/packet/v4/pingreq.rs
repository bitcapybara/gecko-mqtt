use bytes::BytesMut;

use crate::network::packet::Error;

pub struct PingReq;

impl PingReq {
    fn read_from(_stream: &mut BytesMut) -> Result<Self, Error> {
        todo!()
    }
}
