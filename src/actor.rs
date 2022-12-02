use anyhow::Result;
use futures_core::Future;
use tokio::{spawn, task::JoinHandle};

pub trait Actor {
    type Output: Send + Sync + 'static;
    type Future: Future<Output = Result<Self::Output>> + Send + 'static
    where
        Self: 'static,
        Self::Output: 'static;
    fn task(self) -> Self::Future;
    fn spawn(self) -> JoinHandle<Result<Self::Output>>
    where
        Self: Sized + 'static,
    {
        spawn(self.task())
    }
}
pub trait Message {}
pub trait Handler<Message: self::Message> {
    type Reply;
    type Future<'lt>: Future<Output = Result<&'lt Self::Reply>>
    where
        Self: 'lt,
        Self::Reply: 'lt;
    fn handle(&mut self, message: Message) -> Self::Future<'_>;
}
