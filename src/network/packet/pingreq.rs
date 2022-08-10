use bytes::BytesMut;

use super::Error;

pub(crate) struct PingReq;

impl PingReq {
    pub(crate) fn read_from(_stream: &mut BytesMut) -> Result<Self, Error> {
        todo!()
    }
}
