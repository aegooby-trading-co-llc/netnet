use std::net::SocketAddr;

use anyhow::Result;
use futures_util::stream::StreamExt;
use quinn::Endpoint;
use rcgen::generate_simple_self_signed;
use socket2::{Domain, Protocol, SockAddr, Socket, Type};
use tokio::{net::UdpSocket, try_join};
use tokio_util::udp::UdpFramed;
use uuid::Uuid;

use crate::{
    actor::Actor,
    cert::{quic_client_config, quic_server_config},
    codec::Codec,
    peers::PeerTable,
    ping::{PingSink, PingStream},
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
    quic: Quic,
    peers: PeerTable,
}
impl Node {
    pub async fn new(port: u16) -> Result<Self> {
        let framed = UdpFramed::new(UdpSocket::from_std(socket_2(port)?.into())?, Codec::new());
        let (sink, stream) = framed.split();

        let cert = generate_simple_self_signed(vec!["i".into()])?;
        let id = Uuid::new_v5(&Uuid::NAMESPACE_X500, cert.get_key_pair().public_key_raw());

        let mut endpoint = Endpoint::server(
            quic_server_config(&cert)?,
            "0.0.0.0:0".parse::<SocketAddr>()?,
        )?;
        let quic_port = endpoint.local_addr()?.port();
        endpoint.set_default_client_config(quic_client_config(&cert)?);

        let quic = Quic::new(endpoint)?;
        let peers = PeerTable::new(id, quic.senders())?;
        let ping_sink = PingSink::new(sink, id, port, quic_port)?;
        let ping_stream = PingStream::new(stream, id, peers.senders())?;

        Ok(Self {
            ping_sink,
            ping_stream,
            peers,
            quic,
        })
    }

    pub async fn spawn(self) -> Result<()> {
        try_join!(
            self.ping_sink.spawn("ping::sink"),
            self.ping_stream.spawn("ping::stream"),
            self.peers.spawn("peers"),
            self.quic.spawn("quic"),
        )?;
        Ok(())
    }
}
