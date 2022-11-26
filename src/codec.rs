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
        Ok(Some(self::Ping::decode(src)?))
    }
}

impl Encoder<Ping> for Codec {
    type Error = Error;

    fn encode(&mut self, item: Ping, dst: &mut BytesMut) -> Result<(), Self::Error> {
        Ok(item.encode_length_delimited(dst)?)
    }
}

#[cfg(test)]
mod test {
    use anyhow::{Error, Result};
    use bytes::BytesMut;
    use tokio_util::codec::{Decoder, Encoder};

    use super::{Codec, Ping};

    #[tokio::test]
    async fn codec() -> Result<()> {
        let mut codec = Codec::new();
        let mut buffer = BytesMut::new();
        let ping = Ping {
            port: 8080,
            uuid: "penis".into(),
        };
        codec.encode(ping.clone(), &mut buffer)?;
        assert!(
            codec
                .decode(&mut buffer)?
                .map_or(Err(Error::msg("Prost failed to decode buffer")), |item| Ok(
                    item
                ))?
                == ping
        );
        Ok(())
    }
}
