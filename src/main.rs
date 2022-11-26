use std::{collections::HashMap, str::from_utf8, sync::Arc, time::Duration};

use anyhow::{Ok, Result};
use tokio::{
    join, main, spawn,
    sync::Mutex,
    time::{sleep, sleep_until, Instant},
};
use uuid::Uuid;

mod codec;
mod node;
mod verification;

pub mod proto {
    pub mod ping {
        include!(concat!(env!("OUT_DIR"), "/proto.ping.rs"));
    }
}

use node::Node;

use crate::proto::ping::Ping;

#[main(flavor = "multi_thread")]
pub async fn main() -> Result<()> {
    let port = 8080u16;
    // let node = Arc::new(Node::new(port).await?);
    let mut node = Node::new(port).await?;
    // let peers = Arc::new(Mutex::new(HashMap::<Uuid, Instant>::new()));
    // println!("running on port: {}", node.socket.local_addr()?.port());
    // loop {
    node.ping_task().await?;
    // }
    Ok(())
}
