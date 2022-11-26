use anyhow::Result;
use tokio::main;

mod codec;
mod node;
mod verification;

mod proto {
    pub mod ping {
        include!(concat!(env!("OUT_DIR"), "/proto.ping.rs"));
    }
}

use node::Node;

#[main(flavor = "multi_thread")]
pub async fn main() -> Result<()> {
    let mut node = Node::new(8080u16).await?;
    loop {
        node.ping_task().await?;
    }
}
