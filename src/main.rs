#![feature(associated_type_defaults)]
#![feature(tuple_trait)]
#![feature(type_alias_impl_trait)]

use std::{
    env::{set_var, var},
    net::SocketAddr,
};

use anyhow::Result;
use clap::Parser;
use console_subscriber::Builder;
use tokio::main;
use tracing_subscriber::fmt::init;

mod actor;
mod cert;
mod codec;
mod node;
mod peers;
mod ping;
mod quic;
mod util;

mod proto {
    pub mod ping {
        include!(concat!(env!("OUT_DIR"), "/proto.ping.rs"));
    }
}

use crate::node::Node;

#[derive(Parser, Debug)]
struct Args {
    #[arg(long)]
    trace: Option<u16>,
}

#[main(flavor = "multi_thread")]
pub async fn main() -> Result<()> {
    if var("RUST_LOG").is_err() {
        if cfg!(debug) {
            set_var("RUST_LOG", "debug");
        } else {
            set_var("RUST_LOG", "error");
        }
    }
    let args = Args::parse();
    match args.trace {
        Some(port) => Builder::default()
            .server_addr(format!("127.0.0.1:{}", port).parse::<SocketAddr>()?)
            .init(),
        None => init(),
    }

    let node = Node::new(8080u16).await?;
    node.ping_task().await?;
    Ok(())
}
