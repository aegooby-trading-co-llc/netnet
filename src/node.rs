use std::{
    net::{Ipv4Addr, SocketAddr, SocketAddrV4},
    sync::Arc,
    time::Duration,
};

use anyhow::{Ok, Result};
use futures_util::{
    sink::SinkExt,
    stream::{SplitSink, SplitStream, StreamExt},
};
use quinn::Endpoint;
use socket2::{Domain, Protocol, SockAddr, Socket, Type};
use tokio::{net::UdpSocket, spawn, sync::Mutex, time::sleep};
use tokio_util::udp::UdpFramed;
use tracing::debug;
use uuid::Uuid;

use crate::{codec::Codec, proto::ping::Ping, verification::get_server_config};

pub struct Node {
    pub sink: Arc<Mutex<SplitSink<UdpFramed<Codec>, (Ping, SocketAddr)>>>,
    pub stream: Arc<Mutex<SplitStream<UdpFramed<Codec>>>>,
    pub id: Uuid,
    pub endpoint: Endpoint,
    pub port: u16,
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

        let framed = UdpFramed::new(UdpSocket::from_std(socket_2.into())?, Codec::new());
        let port = framed.get_ref().local_addr()?.port();
        let (sink, stream) = framed.split();
        Ok(Self {
            stream: Arc::new(Mutex::new(stream)),
            sink: Arc::new(Mutex::new(sink)),
            id: Uuid::new_v4(),
            endpoint: Endpoint::server(
                get_server_config().await?,
                "127.0.0.1:0".parse::<SocketAddr>()?,
            )?,
            port,
        })
    }
    pub async fn ping_task(&mut self) -> Result<()> {
        let port = self.port;
        let uuid = self.id.clone();

        loop {
            {
                let cloned = self.sink.clone();
                let mut sink = cloned.lock_owned().await;
                spawn(async move {
                    let broadcasthost = format!("255.255.255.255:{}", port);
                    sink.send((
                        Ping {
                            port: port.into(),
                            uuid: uuid.as_hyphenated().to_string(),
                        },
                        broadcasthost.parse()?,
                    ))
                    .await?;
                    sleep(Duration::from_millis(1500)).await;
                    Ok(())
                });
            }

            {
                let cloned = self.stream.clone();
                let mut stream = cloned.lock_owned().await;
                spawn(async move {
                    debug!("read");
                    if let Some(result) = stream.next().await {
                        let (ping, _) = result?;
                        if ping.uuid != uuid.as_hyphenated().to_string() {
                            println!("{:#?}", ping);
                        }
                    }
                    Ok(())
                });
            }
            // (send?, recv?);
        }
        // Ok(())
    }
}
