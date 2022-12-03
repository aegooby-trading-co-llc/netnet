use std::collections::HashMap;

use anyhow::{Error, Result};
use futures_core::Future;
use tokio::{
    spawn,
    sync::mpsc,
    time::{sleep_until, Instant},
};
use tracing::debug;
use uuid::Uuid;

use crate::actor::{Actor, Handler};

#[derive(Copy, Clone, Debug)]
pub struct Peer {
    pub port: u32,
    pub timeout: Instant,
}

pub struct PeerTable {
    peers: HashMap<Uuid, Peer>,
    send: mpsc::Sender<(Uuid, Peer)>,
    recv: mpsc::Receiver<(Uuid, Peer)>,
}
impl PeerTable {
    pub fn new() -> Result<Self> {
        let (sender, receiver) = mpsc::channel(64);
        Ok(Self {
            peers: HashMap::<Uuid, Peer>::new(),
            send: sender,
            recv: receiver,
        })
    }
}
impl Actor for PeerTable {
    type Senders = mpsc::Sender<(Uuid, Peer)>;
    type Future = impl Future<Output = Result<Self::Output>>;

    fn senders(&self) -> Self::Senders {
        self.send.clone()
    }
    fn task(mut self) -> Self::Future {
        async move {
            while let Some(message) = self.recv.recv().await {
                self.handle(message).await?;
            }
            Ok(())
        }
    }
}
impl Handler<(Uuid, Peer)> for PeerTable {
    type Future<'lt> = impl Future<Output = Result<&'lt Self::Reply>>;

    fn handle(&mut self, message: (Uuid, Peer)) -> Self::Future<'_> {
        async move {
            let (id, peer) = message;
            if let Some(existing_peer) = self.peers.get(&id) {
                if existing_peer.timeout == peer.timeout {
                    self.peers.remove(&id);
                    debug!("{:#?}", self.peers);
                    return Ok(&());
                }
            }
            self.peers.insert(id, peer);
            debug!("{:#?}", self.peers);
            let send = self.send.clone();
            spawn(async move {
                sleep_until(peer.timeout).await;
                send.send((id, peer)).await?;
                Ok::<(), Error>(())
            });
            Ok(&())
        }
    }
}
