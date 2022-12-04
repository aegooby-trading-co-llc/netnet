use std::net::SocketAddr;

use anyhow::Result;
use futures_core::Future;
use quinn::{Connecting, Endpoint};
use tokio::{
    select,
    sync::mpsc::{channel, Receiver, Sender},
};

use crate::actor::{Actor, Handler};

#[derive(Clone, Copy, Debug)]
pub struct QuicTarget {
    pub port: u16,
    pub addr: SocketAddr,
}

pub struct Quic {
    endpoint: Endpoint,
    send: Sender<QuicTarget>,
    recv: Receiver<QuicTarget>,
}
impl Quic {
    pub fn new(endpoint: Endpoint) -> Result<Self> {
        let (sender, receiver) = channel(64);
        Ok(Self {
            endpoint,
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
            conn.open_bi().await?;
            Ok(&())
        }
    }
}
impl Handler<QuicTarget> for Quic {
    type Future<'lt> = impl Future<Output = Result<&'lt Self::Reply>>;

    fn handle(&mut self, message: QuicTarget) -> Self::Future<'_> {
        async move {
            let conn = self.endpoint.connect(message.addr, "localhost")?.await?;
            conn.accept_bi().await?;
            Ok(&())
        }
    }
}
