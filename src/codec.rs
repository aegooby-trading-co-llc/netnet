use anyhow::Error;
use tokio_util::codec::{Decoder, Encoder};

use crate::proto;

pub struct Codec;

impl Decoder for Codec {
    type Item = proto::ping::Ping;
    type Error = Error;

    fn decode(&mut self, src: &mut bytes::BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        todo!()
    }
}

impl<Item> Encoder<Item> for Codec {
    type Error = Error;

    fn encode(&mut self, item: Item, dst: &mut bytes::BytesMut) -> Result<(), Self::Error> {
        todo!()
    }
}
