use std::sync::Arc;
use crate::bot::Bot;

#[derive(Debug)]
pub enum LoginResponse {
    /// Login success.
    Success(Arc<Bot>),
    /// Login failed.
    Fail(anyhow::Error),
}