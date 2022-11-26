use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};

use anyhow::{Ok, Result};
use futures_util::{sink::SinkExt, stream::StreamExt};
use quinn::Endpoint;
use socket2::{Domain, Protocol, SockAddr, Socket, Type};
use tokio::net::UdpSocket;
use tokio_util::udp::UdpFramed;
use uuid::Uuid;

use crate::{codec::Codec, proto::ping::Ping, verification::get_server_config};

pub struct Node {
    pub stream: UdpFramed<Codec>,
    pub id: Uuid,
    pub endpoint: Endpoint,
}
impl Node {
    pub async fn new(port: u16) -> Result<Self> {
        let socket_2 = Socket::new(Domain::IPV4, Type::DGRAM, Some(Protocol::UDP))?;
        socket_2.set_reuse_address(true)?;
        socket_2.set_reuse_port(true)?;
        socket_2.bind(&SockAddr::from(SocketAddrV4::new(
            Ipv4Addr::new(0, 0, 0, 0),
            port,
        )))?;
        socket_2.set_broadcast(true)?;
        let config = get_server_config().await?;
        let server_addr = "127.0.0.1:5001".parse::<SocketAddr>()?;

        Ok(Self {
            stream: UdpFramed::new(UdpSocket::from_std(socket_2.into())?, Codec::new()),
            id: Uuid::new_v4(),
            endpoint: Endpoint::server(config, server_addr)?,
        })
    }
    pub async fn ping(&mut self, ping: Ping) -> Result<()> {
        let broadcasthost = format!(
            "255.255.255.255:{}",
            self.stream.get_ref().local_addr()?.port()
        );
        self.stream.send((ping, broadcasthost.parse()?)).await?;
        Ok(())
    }
    pub async fn recv(&self, buffer: &mut [u8]) -> Result<()> {
        // Ok(self.socket.recv_from(buffer).await?)
        Ok(())
    }
    fn port(&self) -> Result<u16> {
        Ok(self.stream.get_ref().local_addr()?.port())
    }
    // pub async fn connect(&self) {
    //     let connection = self.endpoint.connect(server_addr(), SERVER_NAME)?.await?;
    // }
}
