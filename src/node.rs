use std::net::SocketAddr;

use anyhow::Result;
use futures_util::stream::StreamExt;
use quinn::Endpoint;
use socket2::{Domain, Protocol, SockAddr, Socket, Type};
use tokio::{net::UdpSocket, try_join};
use tokio_util::udp::UdpFramed;
use uuid::Uuid;

use crate::{
    actor::Actor,
    cert::{configure_client, get_server_config},
    codec::Codec,
    peers::PeerTable,
    ping::{PingSink, PingStream},
    question,
    quic::Quic,
};

fn socket_2(port: u16) -> Result<Socket> {
    let socket_2 = Socket::new(Domain::IPV4, Type::DGRAM, Some(Protocol::UDP))?;
    socket_2.set_reuse_address(true)?;
    socket_2.set_reuse_port(true)?;
    socket_2.bind(&SockAddr::from(
        format!("0.0.0.0:{port}").parse::<SocketAddr>()?,
    ))?;
    socket_2.set_broadcast(true)?;
    socket_2.set_nonblocking(true)?;
    Ok(socket_2)
}

pub struct Node {
    ping_sink: PingSink,
    ping_stream: PingStream,
    peers: PeerTable,
    quic: Quic,
}
impl Node {
    pub async fn new(port: u16) -> Result<Self> {
        let framed = UdpFramed::new(UdpSocket::from_std(socket_2(port)?.into())?, Codec::new());
        let (sink, stream) = framed.split();
        let mut endpoint = Endpoint::server(
            get_server_config().await?,
            "127.0.0.1:0".parse::<SocketAddr>()?,
        )?;
        let quic_port = endpoint.local_addr()?.port();
        endpoint.set_default_client_config(configure_client());
        let id = Uuid::new_v4();
        let peers = PeerTable::new()?;

        Ok(Self {
            ping_sink: PingSink::new(sink, id, port, quic_port)?,
            ping_stream: PingStream::new(stream, id, peers.senders().clone())?,
            peers,
            quic: Quic::new(endpoint)?,
        })
    }

    pub async fn ping_task(self) -> Result<()> {
        let (sink, stream, peers, quic) = try_join!(
            self.ping_sink.spawn(),
            self.ping_stream.spawn(),
            self.peers.spawn(),
            self.quic.spawn(),
        )?;
        question!(sink, stream, peers, quic);
        Ok(())
    }
}
