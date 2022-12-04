use anyhow::Result;
use tokio::task::JoinHandle;

#[macro_export]
macro_rules! question {
    ($($result:expr),*) => {{
        $($result?;)*
    }};
}

pub async fn yank_handle<Output>(handle: JoinHandle<Result<Output>>) -> Result<Output> {
    Ok(handle.await??)
}
