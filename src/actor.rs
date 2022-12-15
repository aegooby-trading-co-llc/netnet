use std::fmt::Debug;

use anyhow::Result;
use colored::Colorize;
use futures_core::Future;
use tokio::task::{Builder, JoinHandle};
use tracing::debug;

pub trait Actor {
    type Output: Send + Sync = ();
    type Future<'lt>: Future<Output = Result<&'lt Self::Output>> + Send
    where
        Self: 'lt,
        Self::Output: 'lt;
    type Senders: Clone + Debug = ();
    fn senders(&self) -> Self::Senders;
    fn task(&mut self) -> Self::Future<'_>;
    fn shutdown(&mut self, name: String)
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
                let result = match self.task().await {
                    Ok(_) => Ok(()),
                    Err(error) => Err(error),
                };
                self.shutdown(name);
                result
            })?)
    }
}
pub trait Handler<Message: Debug> {
    type Reply = ();
    async fn handle(&mut self, message: Message) -> Result<()>;
}
