use std::{collections::HashMap, net::SocketAddr, sync::Arc, time::Duration};

use anyhow::{Ok, Result};
use futures_util::{
    sink::SinkExt,
    stream::{SplitSink, SplitStream, StreamExt},
};
use quinn::Endpoint;
use socket2::{Domain, Protocol, SockAddr, Socket, Type};
use tokio::{
    join,
    net::UdpSocket,
    spawn,
    sync::Mutex,
    time::{interval, Instant},
};
use tokio_util::udp::UdpFramed;
use tracing::debug;
use uuid::Uuid;

use crate::{
    codec::Codec,
    proto::ping::Ping,
    verification::{configure_client, get_server_config},
};

#[derive(Copy, Clone, Debug)]
pub struct Peer {
    pub port: u32,
    pub timeout: Instant,
}

type Sink = SplitSink<UdpFramed<Codec>, (Ping, SocketAddr)>;
type Stream = SplitStream<UdpFramed<Codec>>;

pub struct Node {
    pub sink: Sink,
    pub stream: Stream,
    // pub sink: Arc<Mutex<Sink>>,
    // pub stream: Arc<Mutex<Stream>>,
    pub id: Uuid,
    pub endpoint: Endpoint,
    pub ping_port: u16,
    pub quic_port: u16,
    pub peers: Arc<Mutex<HashMap<Uuid, Peer>>>,
}
impl Node {
    pub async fn new(port: u16) -> Result<Self> {
        let socket_2 = Socket::new(Domain::IPV4, Type::DGRAM, Some(Protocol::UDP))?;
        socket_2.set_reuse_address(true)?;
        socket_2.set_reuse_port(true)?;
        socket_2.bind(&SockAddr::from(
            format!("0.0.0.0:{}", port).parse::<SocketAddr>()?,
        ))?;
        socket_2.set_broadcast(true)?;

        let framed = UdpFramed::new(UdpSocket::from_std(socket_2.into())?, Codec::new());
        let ping_port = framed.get_ref().local_addr()?.port();
        let (sink, stream) = framed.split();
        let mut endpoint = Endpoint::server(
            get_server_config().await?,
            "127.0.0.1:0".parse::<SocketAddr>()?,
        )?;
        let quic_port = endpoint.local_addr()?.port();
        endpoint.set_default_client_config(configure_client());

        Ok(Self {
            stream,
            sink,
            // stream: Arc::new(Mutex::new(stream)),
            // sink: Arc::new(Mutex::new(sink)),
            id: Uuid::new_v4(),
            ping_port,
            quic_port,
            endpoint,
            peers: Arc::new(Mutex::new(HashMap::<Uuid, Peer>::new())),
        })
    }
    // async fn ping_sink(&mut self) -> Result<()> {
    //     let ping_port = self.ping_port;
    //     let quic_port = self.quic_port;
    //     let uuid = self.id;

    //     let cloned = self.sink.clone();
    //     let mut sink = cloned.lock_owned().await;
    //     let peers = self.peers.clone();
    //     spawn(async move {
    //         {
    //             debug!("peers: {:#?}", peers.try_lock()?);
    //         }
    //         let broadcasthost = format!("255.255.255.255:{}", ping_port);
    //         sink.send((
    //             Ping {
    //                 port: quic_port.into(),
    //                 uuid: uuid.as_hyphenated().to_string(),
    //             },
    //             broadcasthost.parse()?,
    //         ))
    //         .await?;
    //         sleep(Duration::from_millis(1500)).await;
    //         Ok(())
    //     });
    //     Ok(())
    // }
    // async fn ping_stream(&mut self) -> Result<()> {
    //     let uuid = self.id;

    //     let cloned = self.stream.clone();
    //     let peers = self.peers.clone();
    //     let mut stream = cloned.lock_owned().await;
    //     spawn(async move {
    //         if let Some(result) = stream.next().await {
    //             let (ping, _addr) = result?;
    //             if ping.uuid != uuid.as_hyphenated().to_string() {
    //                 debug!("{:#?}", ping);
    //                 let id = Uuid::parse_str(ping.uuid.as_str())?;
    //                 let peer = Peer {
    //                     port: ping.port,
    //                     timeout: Instant::now() + Duration::from_secs(10),
    //                 };
    //                 {
    //                     peers.lock().await.insert(id, peer);
    //                 }
    //                 spawn(async move {
    //                     sleep_until(peer.timeout).await;
    //                     let mut peers = peers.lock().await;
    //                     if let Some(expiry) = peers.get(&id) {
    //                         if expiry.timeout == peer.timeout {
    //                             peers.remove(&id);
    //                         }
    //                     }
    //                 });
    //             }
    //         }
    //         Ok(())
    //     });
    //     Ok(())
    // }
    // pub async fn ping_task(&mut self) -> Result<()> {
    //     loop {
    //         self.ping_sink().await?;
    //         self.ping_stream().await?;
    //     }
    // }
    pub async fn ping_task(self) -> Result<()> {
        let ping_port = self.ping_port;
        let quic_port = self.quic_port;
        let uuid = self.id;

        let mut sink = self.sink;
        let mut stream = self.stream;

        let _ = join!(
            spawn(async move {
                // {
                //     debug!("peers: {:#?}", peers.try_lock()?);
                // }
                let mut i = interval(Duration::from_millis(1000));
                loop {
                    i.tick().await;
                    let broadcasthost = format!("255.255.255.255:{}", ping_port);
                    sink.send((
                        Ping {
                            port: quic_port.into(),
                            uuid: uuid.as_hyphenated().to_string(),
                        },
                        broadcasthost.parse()?,
                    ))
                    .await?;
                    if false {
                        break Ok(());
                    }
                }
            }),
            spawn(async move {
                while let Some(result) = stream.next().await {
                    let (ping, _addr) = result?;
                    if ping.uuid != uuid.as_hyphenated().to_string() {
                        debug!("{:#?}", ping);
                        // let id = Uuid::parse_str(ping.uuid.as_str())?;
                        // let peer = Peer {
                        //     port: ping.port,
                        //     timeout: Instant::now() + Duration::from_secs(10),
                        // };
                        // {
                        //     peers.lock().await.insert(id, peer);
                        // }
                        // spawn(async move {
                        //     sleep_until(peer.timeout).await;
                        //     let mut peers = peers.lock().await;
                        //     if let Some(expiry) = peers.get(&id) {
                        //         if expiry.timeout == peer.timeout {
                        //             peers.remove(&id);
                        //         }
                        //     }
                        // });
                    }
                }
                Ok(())
            })
        );
        Ok(())
    }

    // pub async fn quic_connect()
}
