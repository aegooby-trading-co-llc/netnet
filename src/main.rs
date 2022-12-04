#![feature(associated_type_defaults)]
#![feature(backtrace_frames)]
#![feature(tuple_trait)]
#![feature(type_alias_impl_trait)]

use std::{env::set_var, net::SocketAddr};

use anyhow::Result;
use clap::Parser;
use colored::Colorize;
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
    debug: Option<u16>,
    #[arg(long)]
    bt: bool,
}

async fn __main(args: &Args) -> Result<()> {
    if cfg!(debug) {
        set_var("RUST_LOG", "debug");
        set_var("RUST_LIB_BACKTRACE", "1");
        set_var("RUST_BACKTRACE", "1");
    } else {
        set_var("RUST_LOG", "error");
        set_var("RUST_LIB_BACKTRACE", "0");
        set_var("RUST_BACKTRACE", "1");
    }
    match args.debug {
        Some(port) => Builder::default()
            .server_addr(format!("127.0.0.1:{}", port).parse::<SocketAddr>()?)
            .init(),
        None => init(),
    }

    let node = Node::new(8080u16).await?;
    node.ping_task().await?;
    Ok(())
}

#[main(flavor = "multi_thread")]
pub async fn main() {
    let args = Args::parse();
    if let Err(error) = __main(&args).await {
        println!("{} {}", "error:".bold().red(), error);
        if args.bt {
            println!("{}", error.backtrace());
        }
    }
}
