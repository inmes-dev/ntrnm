pub mod register;
pub mod register_old;
pub mod wtlogin;
mod status;

/// timeout不可以小于5s时间，否则可能导致内存泄露
#[macro_export]
macro_rules! await_response {
    ($timeout:expr, $future:expr, $success_handler:expr, $error_handler:expr) => {{
        match tokio::time::timeout($timeout, $future).await {
            Ok(result) => match result {
                Ok(value) => $success_handler(value),
                Err(e) => $error_handler(e),
            },
            Err(_) => $error_handler(anyhow::Error::msg("Timeout occurred")),
        }
    }};
}
