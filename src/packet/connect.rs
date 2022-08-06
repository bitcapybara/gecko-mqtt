use bytes::BytesMut;

use crate::error::Result;

pub(crate) struct Connect {}

impl Connect {
    pub(crate) fn read_from(_stream: &mut BytesMut) -> Result<Self> {
        todo!()
    }
}
