use std::{
    net::{Ipv4Addr, SocketAddr, SocketAddrV4},
    sync::Arc,
};

use anyhow::Error;
use socket2::{Domain, Protocol, SockAddr, Socket, Type};
use tokio::net::UdpSocket;
use uuid::Uuid;

#[derive(Clone)]
pub struct Node {
    pub socket: Arc<UdpSocket>,
    pub id: Uuid,
}
impl Node {
    pub fn new(port: u16) -> Result<Self, Error> {
        let socket_2 = Socket::new(Domain::IPV4, Type::DGRAM, Some(Protocol::UDP))?;
        socket_2.set_reuse_address(true)?;
        socket_2.set_reuse_port(true)?;
        socket_2.bind(&SockAddr::from(SocketAddrV4::new(
            Ipv4Addr::new(0, 0, 0, 0),
            port,
        )))?;
        socket_2.set_broadcast(true)?;
        Ok(Self {
            socket: Arc::new(UdpSocket::from_std(socket_2.into())?),
            id: Uuid::new_v4(),
        })
    }
    pub async fn send(&self, buffer: &[u8]) -> Result<usize, Error> {
        let broadcasthost = format!("255.255.255.255:{}", self.socket.local_addr()?.port());
        Ok(self.socket.send_to(buffer, broadcasthost).await?)
    }
    pub async fn recv(&self, buffer: &mut [u8]) -> Result<(usize, SocketAddr), Error> {
        Ok(self.socket.recv_from(buffer).await?)
    }
}
