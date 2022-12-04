use std::fmt::Debug;

use anyhow::Result;
use futures_core::Future;
use tokio::task::{Builder, JoinHandle};

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
    fn spawn(mut self, name: &'static str) -> Result<JoinHandle<Result<()>>>
    where
        Self: Sized + Send + Sync + 'static,
    {
        Ok(Builder::default().name(name).spawn(async move {
            let result = match self.task().await {
                Ok(_) => Ok(()),
                Err(error) => Err(error),
            };
            self.shutdown();
            result
        })?)
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
