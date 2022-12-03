use std::fmt::Debug;

use anyhow::Result;
use futures_core::Future;
use tokio::{spawn, task::JoinHandle};

pub trait Actor {
    type Output: Send + Sync = ();
    type Future<'lt>: Future<Output = Result<&'lt Self::Output>> + Send
    where
        Self: 'lt,
        Self::Output: 'lt;
    type Senders: Clone + Debug = ();
    fn senders(&self) -> Self::Senders;
    fn task(&mut self) -> Self::Future<'_>;
    fn shutdown(&mut self)
    where
        Self: Sized,
    {
    }
    fn spawn(mut self) -> JoinHandle<Result<()>>
    where
        Self: Sized + Send + Sync + 'static,
    {
        let handle = spawn(async move {
            self.task().await?;
            self.shutdown();
            Ok(())
        });
        handle
    }
}
// pub trait Message: Clone + Debug {}
pub trait Handler<Message: Clone + Debug> {
    type Reply = ();
    type Future<'lt>: Future<Output = Result<&'lt Self::Reply>>
    where
        Self: 'lt,
        Self::Reply: 'lt;
    fn handle(&mut self, message: Message) -> Self::Future<'_>;
}
