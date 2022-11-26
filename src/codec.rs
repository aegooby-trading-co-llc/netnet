use anyhow::Error;
use bytes::BytesMut;
use tokio_util::codec::{Decoder, Encoder};

use crate::proto;

pub struct Codec;

impl Decoder for Codec {
    type Item = proto::ping::Ping;
    type Error = Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        todo!()
    }
}

impl<Item> Encoder<Item> for Codec {
    type Error = Error;

    fn encode(&mut self, item: Item, dst: &mut BytesMut) -> Result<(), Self::Error> {
        todo!()
    }
}
