use std::{str::from_utf8, time::Duration};

use anyhow::Error;
use tokio::{
    main,
    time::{interval, sleep},
};

mod node;

use node::Node;

#[main(flavor = "multi_thread")]
pub async fn main() -> Result<(), Error> {
    let node = Node::new(8081)?;
    let reader = node.clone();
    let writer = node.clone();
    println!("running on port: {}", node.socket.local_addr()?.port());
    let (_read, _write) = tokio::join!(
        tokio::spawn(async move {
            loop {
                let mut buf = *b"         ";
                let (_, addr) = reader.recv(&mut buf).await.expect("");
                match addr {
                    std::net::SocketAddr::V4(addr_v4) => {
                        if !addr_v4.ip().is_private() {
                            println!("recv: '{}' from '{}'", from_utf8(&buf).expect(""), addr);
                        }
                    }
                    std::net::SocketAddr::V6(addr_v6) => {}
                }
                println!("recv(all): '{}' from {}", from_utf8(&buf).expect(""), addr);
            }
        }),
        tokio::spawn(async move {
            let mut int = interval(Duration::from_millis(1000));
            loop {
                int.tick().await;
                println!("begin");
                writer.send(b"ping1").await.expect("");
                println!("mid");
                int.tick().await;
                println!("end");
                writer.send(b"ping2").await.expect("");
            }
        })
    );
    Ok(())
}
