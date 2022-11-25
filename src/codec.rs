use anyhow::Error;
use tokio_util::codec::Decoder;

#[derive(Debug)]
pub struct Message;

pub struct Codec;

impl Decoder for Codec {
    type Item = Message;

    type Error = Error;

    fn decode(&mut self, src: &mut bytes::BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        todo!()
    }
}
