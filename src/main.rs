use std::{
    net::{Ipv4Addr, SocketAddrV4},
    str::from_utf8,
    sync::Arc,
};

use anyhow::Error;
use socket2::{Domain, Protocol, SockAddr, Socket, Type};
use tokio::{
    io::{stdin, AsyncBufReadExt, BufReader},
    main,
    net::UdpSocket,
};

#[main]
pub async fn main() -> Result<(), Error> {
    let socket_2 = Socket::new(Domain::IPV4, Type::DGRAM, Some(Protocol::UDP))?;
    socket_2.set_reuse_address(true)?;
    socket_2.set_reuse_port(true)?;
    socket_2.bind(&SockAddr::from(SocketAddrV4::new(
        Ipv4Addr::new(0, 0, 0, 0),
        // 0,
        8080,
    )))?;
    socket_2.set_broadcast(true)?;

    let socket = Arc::new(UdpSocket::from_std(socket_2.into())?);
    println!("running on port: {}", socket.local_addr()?.port());

    let reader = socket.clone();
    let writer = socket.clone();
    let (_read, _write) = tokio::join!(
        tokio::spawn(async move {
            loop {
                let mut buf = *b"     ";
                reader.recv_from(&mut buf).await.expect("");
                println!("buffer: {}", from_utf8(&buf).expect(""));
            }
        }),
        tokio::spawn(async move {
            let stdin = stdin();
            let reader = BufReader::new(stdin);
            let mut lines = reader.lines();
            while let Some(line) = lines.next_line().await.expect("") {
                writer
                    .send_to(
                        line.as_bytes(),
                        format!("255.255.255.255:{}", socket.local_addr().expect("").port()),
                    )
                    .await
                    .expect("");
            }
        })
    );
    Ok(())
}
