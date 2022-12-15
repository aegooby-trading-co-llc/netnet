<<<<<<< Updated upstream
use std::net::SocketAddr;

use anyhow::Result;
use futures_core::Future;
use quinn::{Connecting, Endpoint};
=======
use std::{collections::HashMap, net::SocketAddr};

use anyhow::Result;
use futures_core::Future;
use quinn::{Connecting, Connection, Endpoint};
>>>>>>> Stashed changes
use tokio::{
    select,
    sync::mpsc::{channel, Receiver, Sender},
};
<<<<<<< Updated upstream
=======
use tracing::debug;
>>>>>>> Stashed changes

use crate::actor::{Actor, Handler};

#[derive(Clone, Copy, Debug)]
pub struct QuicTarget {
    pub port: u16,
    pub addr: SocketAddr,
}

pub struct Quic {
    endpoint: Endpoint,
<<<<<<< Updated upstream
=======
    conns: HashMap<SocketAddr, Connection>,
>>>>>>> Stashed changes
    send: Sender<QuicTarget>,
    recv: Receiver<QuicTarget>,
}
impl Quic {
    pub fn new(endpoint: Endpoint) -> Result<Self> {
        let (sender, receiver) = channel(64);
        Ok(Self {
            endpoint,
<<<<<<< Updated upstream
=======
            conns: HashMap::new(),
>>>>>>> Stashed changes
            send: sender,
            recv: receiver,
        })
    }
}
impl Actor for Quic {
    type Future<'lt> = impl Future<Output = Result<&'lt Self::Output>>;
    type Senders = Sender<QuicTarget>;

    fn senders(&self) -> Self::Senders {
        self.send.clone()
    }
    fn task(&mut self) -> Self::Future<'_> {
        async move {
            loop {
                select! {
                    // Incoming connections: our id > peer id
                    Some(message) = self.endpoint.accept() => {
                        self.handle(message).await?;
                    }
                    // Outgoing connections: our id < peer id
                    Some(message) = self.recv.recv() => {
                        self.handle(message).await?;
                    }
                    else => break Ok(&())
                }
            }
        }
    }
}
impl Handler<Connecting> for Quic {
    type Future<'lt> = impl Future<Output = Result<&'lt Self::Reply>>;

    fn handle(&mut self, message: Connecting) -> Self::Future<'_> {
        async move {
            let conn = message.await?;
<<<<<<< Updated upstream
            conn.open_bi().await?;
=======
            debug!("quic: incoming connection successful");
            self.conns.insert(conn.remote_address(), conn.clone());
            debug!("{:#?}", conn.open_bi().await?);
>>>>>>> Stashed changes
            Ok(&())
        }
    }
}
impl Handler<QuicTarget> for Quic {
    type Future<'lt> = impl Future<Output = Result<&'lt Self::Reply>>;

<<<<<<< Updated upstream
    fn handle(&mut self, _message: QuicTarget) -> Self::Future<'_> {
        async move {
            // let conn = self.endpoint.connect(message.addr, "localhost")?.await?;
            // conn.accept_bi().await?;
=======
    fn handle(&mut self, message: QuicTarget) -> Self::Future<'_> {
        async move {
            let conn = self
                .endpoint
                .connect(
                    SocketAddr::new("127.0.0.1".parse()?, message.port),
                    // SocketAddr::new(message.addr.ip(), message.port),
                    "localhost",
                )?
                .await?;
            debug!("quic: outgoing connection successful");
            self.conns.insert(conn.remote_address(), conn.clone());
            debug!("{:#?}", conn.accept_bi().await?);
>>>>>>> Stashed changes
            Ok(&())
        }
    }
}
