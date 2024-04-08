use futures::executor::ThreadPool;
use once_cell::sync::Lazy;

pub static DEFAULT_POOL: Lazy<ThreadPool> = Lazy::new(|| {
    ThreadPool::new().expect("Failed to build thread pool")
});