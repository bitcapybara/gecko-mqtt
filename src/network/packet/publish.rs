use bytes::BytesMut;

use super::Error;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub(crate) struct Publish {}

impl Publish {
    pub(crate) fn read_from(_stream: &mut BytesMut) -> Result<Self, Error> {
        todo!()
    }
}
