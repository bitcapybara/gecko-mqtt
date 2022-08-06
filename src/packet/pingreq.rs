use bytes::BytesMut;

use crate::error::Result;

pub(crate) struct PingReq {}

impl PingReq {
    pub(crate) fn read_from(_stream: &mut BytesMut) -> Result<Self> {
        todo!()
    }
}
