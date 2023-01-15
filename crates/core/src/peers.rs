use std::{collections::HashMap, net::SocketAddr};

use anyhow::{Error, Result};
use futures_core::Future;
use tokio::{
    spawn,
    sync::mpsc::{channel, Receiver, Sender},
    time::{sleep_until, Instant},
};
use tracing::debug;
use uuid::Uuid;

use crate::{
    actor::{Actor, Handler},
    quic::QuicTarget,
};

#[derive(Copy, Clone, Debug)]
pub struct Peer {
    pub addr: SocketAddr,
    pub port: u32,
    pub timeout: Instant,
    pub death: Option<Instant>,
}

pub struct PeerTable {
    peers: HashMap<Uuid, Peer>,
    id: Uuid,
    send: Sender<(Uuid, Peer)>,
    recv: Receiver<(Uuid, Peer)>,
    quic_send: Sender<QuicTarget>,
}
impl PeerTable {
    pub fn new(id: Uuid, quic_send: Sender<QuicTarget>) -> Result<Self> {
        let (sender, receiver) = channel(64);
        Ok(Self {
            peers: HashMap::<Uuid, Peer>::new(),
            id,
            send: sender,
            recv: receiver,
            quic_send,
        })
    }
}
impl Actor for PeerTable {
    type Senders = Sender<(Uuid, Peer)>;
    fn senders(&self) -> Self::Senders {
        self.send.clone()
    }
    fn task(&mut self) -> impl Future<Output = Result<Self::Output>> + Send + '_ {
        async move {
            while let Some(message) = self.recv.recv().await {
                self.handle(message).await?;
            }
            Ok(())
        }
    }
}
impl Handler<(Uuid, Peer)> for PeerTable {
    async fn handle(&mut self, message: (Uuid, Peer)) -> Result<Self::Reply> {
        let (id, new_peer) = message;
        if let Some(old_peer) = self.peers.get_mut(&id) {
            match (old_peer.death, new_peer.death) {
                // If you're already dead stop holding new funerals over and over. People
                // aren't sad anymore, they're just going there to hang out and talk shit
                // about you.
                (Some(_), Some(_)) => return Ok(()),
                (None, Some(_)) => {
                    // If there have been no new pings, it really did die.
                    if old_peer.timeout == new_peer.timeout {
                        debug!("peer(dead): {}", id.as_hyphenated().to_string());
                        old_peer.death = new_peer.death;
                    }
                    // Otherwise, ignore fake deaths.
                    return Ok(());
                }
                (Some(death), None) => {
                    // New live peers always get inserted unless it died after this
                    // message's lifespan (ignore overly old messages).
                    if death > new_peer.timeout {
                        return Ok(());
                    }
                }
                // Alive and well baybee.
                (None, None) => (),
            }
        } else {
            debug!("peer(new): {}", id.as_hyphenated().to_string());
            if self.id < id {
                self.quic_send
                    .send(QuicTarget {
                        addr: new_peer.addr,
                        port: u16::try_from(new_peer.port)?,
                    })
                    .await?;
            }
        }
        self.peers.insert(id, new_peer);
        let send = self.send.clone();
        spawn(async move {
            sleep_until(new_peer.timeout).await;
            let dying_peer = Peer {
                death: Some(new_peer.timeout),
                ..new_peer
            };
            send.send((id, dying_peer)).await?;
            Ok::<(), Error>(())
        });
        Ok(())
    }
}
