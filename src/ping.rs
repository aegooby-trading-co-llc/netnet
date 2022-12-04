use std::{net::SocketAddr, time::Duration};

use anyhow::Result;
use futures_core::Future;
use futures_util::{
    stream::{SplitSink, SplitStream},
    SinkExt, StreamExt,
};
use tokio::{
    sync::mpsc,
    time::{interval, Instant},
};
use tokio_util::udp::UdpFramed;
use uuid::Uuid;

use crate::{
    actor::{Actor, Handler},
    codec::Codec,
    generated::ping::Ping,
    peers::Peer,
};

type Sink = SplitSink<UdpFramed<Codec>, (Ping, SocketAddr)>;
type Stream = SplitStream<UdpFramed<Codec>>;

pub struct PingSink {
    sink: Sink,
    uuid: Uuid,
    port: u16,
    quic_port: u16,
}
impl PingSink {
    pub fn new(sink: Sink, uuid: Uuid, port: u16, quic_port: u16) -> Result<Self> {
        Ok(Self {
            sink,
            uuid,
            port,
            quic_port,
        })
    }
}
impl Actor for PingSink {
    type Future<'lt> = impl Future<Output = Result<&'lt Self::Output>>;

    fn senders(&self) -> Self::Senders {
        ()
    }
    fn task(&mut self) -> Self::Future<'_> {
        async move {
            let mut i = interval(Duration::from_millis(1000));
            loop {
                i.tick().await;
                let message = Ping {
                    port: self.quic_port.into(),
                    uuid: self.uuid.as_hyphenated().to_string(),
                };
                self.handle(message).await?;
                if false {
                    break Ok(&());
                }
            }
        }
    }
}
impl Handler<Ping> for PingSink {
    type Future<'lt> = impl Future<Output = Result<&'lt Self::Reply>>;

    fn handle(&mut self, message: Ping) -> Self::Future<'_> {
        async move {
            let broadcasthost = format!("255.255.255.255:{}", self.port);
            self.sink.send((message, broadcasthost.parse()?)).await?;
            Ok(&())
        }
    }
}

pub struct PingStream {
    stream: Stream,
    uuid: Uuid,
    peers: mpsc::Sender<(Uuid, Peer)>,
}
impl PingStream {
    pub fn new(stream: Stream, uuid: Uuid, peers: mpsc::Sender<(Uuid, Peer)>) -> Result<Self> {
        Ok(Self {
            stream,
            uuid,
            peers,
        })
    }
}
impl Actor for PingStream {
    type Future<'lt> = impl Future<Output = Result<&'lt Self::Output>>;

    fn senders(&self) -> Self::Senders {
        ()
    }
    fn task(&mut self) -> Self::Future<'_> {
        async move {
            while let Some(message) = self.stream.next().await {
                self.handle(message?).await?;
            }
            Ok(&())
        }
    }
}
impl Handler<(Ping, SocketAddr)> for PingStream {
    type Future<'lt> = impl Future<Output = Result<&'lt Self::Reply>>;

    fn handle(&mut self, message: (Ping, SocketAddr)) -> Self::Future<'_> {
        async move {
            let (ping, addr) = message;
            if ping.uuid != self.uuid.as_hyphenated().to_string() {
                let id = Uuid::parse_str(ping.uuid.as_str())?;
                let peer = Peer {
                    addr,
                    port: ping.port,
                    timeout: Instant::now() + Duration::from_secs(10),
                    death: None,
                };
                self.peers.send((id, peer)).await?;
            }
            Ok(&())
        }
    }
}
