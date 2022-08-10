use bytes::BytesMut;

use super::{Error, PacketReadWriter};

pub(crate) struct Connect {}

impl PacketReadWriter for Connect {
    fn read_from(_stream: &mut BytesMut) -> Result<Self, Error> {
        todo!()
    }
}
