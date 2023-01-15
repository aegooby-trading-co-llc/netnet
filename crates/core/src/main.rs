#![allow(incomplete_features)]
#![feature(async_fn_in_trait)]
#![feature(associated_type_defaults)]
#![feature(type_alias_impl_trait)]
#![feature(return_position_impl_trait_in_trait)]

use std::env::set_var;
#[cfg(feature = "console")]
use std::net::SocketAddr;

use anyhow::Result;
use clap::Parser;
use colored::Colorize;
#[cfg(feature = "console")]
use console_subscriber::Builder;
use tokio::main;
use tracing_subscriber::fmt::init;

mod actor;
mod cert;
mod codec;
mod gen;
mod node;
mod peers;
mod ping;
mod quic;

use crate::node::Node;

#[derive(Parser, Debug)]
struct Args {
    #[arg(short = 'c', long = "console")]
    console: Option<u16>,
    #[arg(short = 't', long = "trace")]
    trace: bool,
    #[arg(short = 'b', long = "backtrace")]
    backtrace: bool,
}

async fn __main(args: &Args) -> Result<()> {
    if cfg!(debug) {
        set_var("RUST_LOG", if args.trace { "trace" } else { "debug" });
        set_var("RUST_LIB_BACKTRACE", "1");
        set_var("RUST_BACKTRACE", "1");
    } else {
        set_var("RUST_LOG", "error");
        set_var("RUST_LIB_BACKTRACE", "0");
        set_var("RUST_BACKTRACE", "1");
    }
    match args.console {
        #[cfg(feature = "console")]
        Some(port) => Builder::default()
            .server_addr(format!("127.0.0.1:{port}").parse::<SocketAddr>()?)
            .init(),
        _ => init(),
    }

    let node = Node::new(8080u16).await?;
    node.spawn().await?;
    Ok(())
}

#[main(flavor = "multi_thread")]
pub async fn main() {
    let args = Args::parse();
    if let Err(error) = __main(&args).await {
        eprintln!("{} {}", "error:".bold().red(), error);
        if args.backtrace {
            eprintln!("{}", error.backtrace());
        }
    }
}
