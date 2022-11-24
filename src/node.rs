use std::{
    net::{Ipv4Addr, SocketAddr, SocketAddrV4},
    sync::Arc,
};

use quinn::{Endpoint}; 

use anyhow::Error;
use socket2::{Domain, Protocol, SockAddr, Socket, Type};
use tokio::net::UdpSocket;
use uuid::Uuid;

use crate::verification;


#[derive(Clone)]
pub struct Node {
    pub socket: Arc<UdpSocket>,
    pub id: Uuid,
    pub endpoint:Endpoint
}
impl Node {
    pub async fn new(port: u16) -> Result<Self, Error> {
        let socket_2 = Socket::new(Domain::IPV4, Type::DGRAM, Some(Protocol::UDP))?;
        socket_2.set_reuse_address(true)?;
        socket_2.set_reuse_port(true)?;
        socket_2.bind(&SockAddr::from(SocketAddrV4::new(
            Ipv4Addr::new(0, 0, 0, 0),
            port,
        )))?;
        socket_2.set_broadcast(true)?;
        let config = verification::get_server_config().await?;
        let server_addr = "127.0.0.1:5001".parse::<SocketAddr>()?; 
        Ok(Self {
            socket: Arc::new(UdpSocket::from_std(socket_2.into())?),
            id: Uuid::new_v4(),
            endpoint: Endpoint::server(config, server_addr)?
        })
    }
    pub async fn send(&self, buffer: &[u8]) -> Result<usize, Error> {
        let broadcasthost = format!("255.255.255.255:{}", self.socket.local_addr()?.port());
        Ok(self.socket.send_to(buffer, broadcasthost).await?)
    }
    pub async fn recv(&self, buffer: &mut [u8]) -> Result<(usize, SocketAddr), Error> {
        Ok(self.socket.recv_from(buffer).await?)
    }
    // pub async fn connect(&self) { 
    //     let connection = self.endpoint.connect(server_addr(), SERVER_NAME)?.await?;
    // }
}
