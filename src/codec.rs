use anyhow::Error;
use bytes::BytesMut;
use prost::Message;
use tokio_util::codec::{Decoder, Encoder};

use crate::proto::ping::Ping;

pub struct Codec;
impl Codec {
    pub fn new() -> Self {
        Self
    }
}

impl Decoder for Codec {
    type Item = Ping;
    type Error = Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        todo!()
    }
}

impl Encoder<Ping> for Codec {
    type Error = Error;

    fn encode(&mut self, item: Ping, dst: &mut BytesMut) -> Result<(), Self::Error> {
        dst.reserve(item.encoded_len());
        item.encode(dst)?;
        Ok(())
    }
}
