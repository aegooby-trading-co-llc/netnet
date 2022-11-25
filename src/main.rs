use std::{collections::HashMap, str::from_utf8, sync::Arc, time::Duration};

use anyhow::{Ok, Result};
use tokio::{
    join, main, select,
    signal::ctrl_c,
    spawn,
    sync::Mutex,
    time::{sleep, sleep_until, Instant},
};
use uuid::Uuid;

mod codec;
mod node;
mod verification;

pub mod snazzy {
    pub mod items {
        include!(concat!(env!("OUT_DIR"), "/snazzy.items.rs"));
    }
}

use node::Node;

// struct Peer {
//     timestamp: Instant,
//     port: u16
// }

#[main(flavor = "multi_thread")]
pub async fn main() -> Result<()> {
    select! {
        _ = ctrl_c() => {},
    }

    let node = Node::new(8080).await?;
    let peers = Arc::new(Mutex::new(HashMap::<Uuid, Instant>::new()));
    // println!("running on port: {}", node.socket.local_addr()?.port());
    loop {
        let peers_reader = peers.clone();
        let peers_writer = peers.clone();
        let reader = node.clone();
        let writer = node.clone();
        let _ = join!(
            spawn(async move {
                let mut buf = *b"                                    ";

                let _ = reader.recv(&mut buf).await?;
                let id = Uuid::parse_str(from_utf8(&buf)?)?;
                if id != reader.id {
                    let timestamp = Instant::now() + Duration::from_secs(10);
                    {
                        peers_writer.lock().await.insert(id, timestamp);
                    }
                    spawn(async move {
                        sleep_until(timestamp).await;
                        let mut peers = peers_writer.lock().await;
                        if let Some(expiry) = peers.get(&id) {
                            if *expiry == timestamp {
                                peers.remove(&id);
                            }
                        }
                    });
                }
                Ok::<()>(())
            }),
            spawn(async move {
                writer
                    .send(format!("{}", writer.id.as_hyphenated()).as_bytes())
                    .await?;
                sleep(Duration::from_millis(500)).await;
                Ok::<()>(())
            }),
            spawn(async move {
                println!("peers: {:#?}", peers_reader.lock().await);
                sleep(Duration::from_millis(2000)).await;
                Ok::<()>(())
            })
        );
    }
}
