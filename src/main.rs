use std::{collections::HashMap, str::from_utf8, time::Duration};

use anyhow::Error;
use tokio::{join, main, spawn, time::sleep};

mod node;

use node::Node;
use uuid::Uuid;

#[main(flavor = "multi_thread", worker_threads = 2)]
pub async fn main() -> Result<(), Error> {
    let node = Node::new(8080)?;
    let hashmap = HashMap::<Uuid, String>::new();
    println!("running on port: {}", node.socket.local_addr()?.port());
    loop {
        let reader = node.clone();
        let writer = node.clone();
        let _ = join!(
            spawn(async move {
                let mut buf = *b"                                    ";
                let (_, addr) = reader.recv(&mut buf).await?;
                println!("recv(all): '{}' from {}", from_utf8(&buf)?, addr);
                Ok::<(), Error>(())
            }),
            spawn(async move {
                writer
                    .send(format!("{}", writer.id.as_hyphenated()).as_bytes())
                    .await?;
                sleep(Duration::from_millis(500)).await;
                Ok::<(), Error>(())
            })
        );
    }
}
