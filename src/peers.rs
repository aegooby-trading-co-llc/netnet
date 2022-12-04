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
    pub death: Option<Instant>,
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
    type Future<'lt> = impl Future<Output = Result<&'lt Self::Output>>;

    fn senders(&self) -> Self::Senders {
        self.send.clone()
    }
    fn task(&mut self) -> Self::Future<'_> {
        async move {
            while let Some(message) = self.recv.recv().await {
                self.handle(message).await?;
            }
            Ok(&())
        }
    }
}
impl Handler<(Uuid, Peer)> for PeerTable {
    type Future<'lt> = impl Future<Output = Result<&'lt Self::Reply>>;

    fn handle(&mut self, message: (Uuid, Peer)) -> Self::Future<'_> {
        async move {
            let (id, new_peer) = message;
            if let Some(old_peer) = self.peers.get_mut(&id) {
                match (old_peer.death, new_peer.death) {
                    // If you're already dead stop holding new funerals over and over. People
                    // aren't sad anymore, they're just going there to hang out and talk shit
                    // about you.
                    (Some(_), Some(_)) => return Ok(&()),
                    (None, Some(_)) => {
                        // If there have been no new pings, it really did die.
                        if old_peer.timeout == new_peer.timeout {
                            debug!("Dead peer: {}", id.as_hyphenated().to_string());
                            old_peer.death = new_peer.death;
                        }
                        // Otherwise, ignore fake deaths.
                        return Ok(&());
                    }
                    (Some(death), None) => {
                        // New live peers always get inserted unless it died after this
                        // message's lifespan (ignore overly old messages).
                        if death > new_peer.timeout {
                            return Ok(&());
                        }
                    }
                    // Alive and well baybee.
                    (None, None) => (),
                }
            } else {
                debug!("New peer: {}", id.as_hyphenated().to_string());
            }
            self.peers.insert(id, new_peer);
            let send = self.send.clone();
            spawn(async move {
                sleep_until(new_peer.timeout).await;
                let dying_peer = Peer {
                    death: Some(new_peer.timeout),
                    ..new_peer.clone()
                };
                send.send((id, dying_peer)).await?;
                Ok::<(), Error>(())
            });
            Ok(&())
        }
    }
}
