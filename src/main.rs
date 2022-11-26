use std::env::{set_var, var};

use anyhow::Result;
use console_subscriber::init;
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
    if var("RUST_LOG").is_err() {
        if cfg!(debug) {
            set_var("RUST_LOG", "debug");
        } else {
            set_var("RUST_LOG", "error");
        }
    }
    init();

    let mut node = Node::new(8080u16).await?;
    node.ping_task().await?;
    Ok(())
}
