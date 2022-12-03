use std::fmt::Debug;

use anyhow::Result;
use futures_core::Future;
use tokio::{spawn, task::JoinHandle};

pub trait Actor {
    type Output: Send + Sync + 'static = ();
    type Future: Future<Output = Result<Self::Output>> + Send + 'static
    where
        Self: 'static,
        Self::Output: 'static;
    type Senders: Clone + Debug = ();
    fn senders(&self) -> Self::Senders;
    fn task(self) -> Self::Future;
    fn shutdown(&self)
    where
        Self: Sized + 'static,
    {
    }
    fn spawn(self) -> JoinHandle<Result<Self::Output>>
    where
        Self: Sized + Send + Sync + 'static,
    {
        let handle = spawn(async move {
            let future = self.task().await;
            // self.shutdown();
            future
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
