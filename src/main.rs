use std::{collections::HashMap, str::from_utf8, sync::Arc, time::Duration};

use anyhow::Error;
use tokio::{
    join, main, spawn,
    sync::Mutex,
    time::{sleep, sleep_until, Instant},
};
use uuid::Uuid;

mod node;

use node::Node;

#[main(flavor = "multi_thread")]
pub async fn main() -> Result<(), Error> {
    let node = Node::new(8080)?;
    let peers = Arc::new(Mutex::new(HashMap::<Uuid, Instant>::new()));
    println!("running on port: {}", node.socket.local_addr()?.port());
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
                Ok::<(), Error>(())
            }),
            spawn(async move {
                writer
                    .send(format!("{}", writer.id.as_hyphenated()).as_bytes())
                    .await?;
                sleep(Duration::from_millis(500)).await;
                Ok::<(), Error>(())
            }),
            spawn(async move {
                println!("peers: {:#?}", peers_reader.lock().await);
                sleep(Duration::from_millis(2000)).await;
                Ok::<(), Error>(())
            })
        );
    }
}
