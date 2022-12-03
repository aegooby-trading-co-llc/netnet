use anyhow::Result;
use futures_core::Future;
use quinn::Endpoint;
use tokio::select;

use crate::actor::Actor;

pub struct Quic {
    endpoint: Endpoint,
}
impl Quic {
    pub fn new(endpoint: Endpoint) -> Result<Self> {
        Ok(Self { endpoint })
    }
}
impl Actor for Quic {
    type Future = impl Future<Output = Result<Self::Output>>;
    type Senders = ();

    fn senders(&self) -> Self::Senders {
        ()
    }
    fn task(self) -> Self::Future {
        async move {
            loop {
                if let Some(conn) = self.endpoint.accept().await {
                    let conn = conn.await?;
                }

                select! {
                    Some(accept) = self.endpoint.accept() => {
                        let conn = accept.await?;
                    }
                    else => break Ok(())
                }
            }
        }
    }
}
