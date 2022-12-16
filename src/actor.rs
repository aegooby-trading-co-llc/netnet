use std::fmt::Debug;

use anyhow::Result;
use colored::Colorize;
use futures_core::Future;
use tokio::task::{Builder, JoinHandle};
use tracing::debug;

pub trait Actor {
    type Output: Send + Sync = ();
    type Senders: Clone + Debug = ();
    fn senders(&self) -> Self::Senders;
    fn task(&mut self) -> impl Future<Output = Result<Self::Output>> + Send + '_;
    async fn shutdown(&mut self, name: String)
    where
        Self: Sized,
    {
        debug!("task(shutdown) {}", name.cyan());
    }
    fn spawn(mut self, name: &'static str) -> Result<JoinHandle<Result<()>>>
    where
        Self: Sized + Send + Sync + 'static,
    {
        let name = format!("netnet::{name}");
        debug!("task(spawn) {}", name.cyan());
        Ok(Builder::default()
            .name(name.clone().as_str())
            .spawn(async move {
                let result = self.task().await.map(|_| ());
                self.shutdown(name).await;
                result
            })?)
    }
}
pub trait Handler<Message: Debug> {
    type Reply = ();
    async fn handle(&mut self, message: Message) -> Result<Self::Reply>;
}
