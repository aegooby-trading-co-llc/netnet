#[macro_export]
macro_rules! question {
    ($($result:expr),*) => {{
        $($result?;)*
    }};
}
