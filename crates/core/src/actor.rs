use std::fmt::Debug;

use anyhow::Result;
use colored::Colorize;
use futures_core::Future;
use tokio::task::Builder;
use tracing::debug;

pub trait Actor {
    type Output: Send + Sync = ();
    type Senders: Clone + Debug = ();
    fn senders(&self) -> Self::Senders;
    fn task(&mut self) -> impl Future<Output = Result<Self::Output>> + Send + '_;
    async fn shutdown(&mut self, name: String) {
        debug!("task(shutdown) {}", name.cyan());
    }
    async fn spawn(mut self, name: &'static str) -> Result<()>
    where
        Self: Sized + Send + 'static,
    {
        let name = format!("netnet::{name}");
        debug!("task(spawn) {}", name.cyan());
        Builder::default()
            .name(name.clone().as_str())
            .spawn(async move {
                let result = self.task().await.map(|_| ());
                self.shutdown(name).await;
                result
            })?
            .await?
    }
}
pub trait Handler<Message: Debug> {
    type Reply = ();
    async fn handle(&mut self, message: Message) -> Result<Self::Reply>;
}
