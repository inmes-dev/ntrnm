use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, Mutex, OnceLock};
use std::sync::atomic::{AtomicBool, Ordering};
use std::process::exit;
use std::thread;

use tokio::runtime;
use crate::tokiort::global_tokio_runtime;

pub struct SigintHandler<F>
    where
        F: Future + Send + ?Sized + 'static,
        F::Output: 'static
{
    listeners: Arc<Mutex<Vec<Pin<Box<F>>>>>
}

pub fn global_sigint_handler() -> Arc<SigintHandler<dyn Future<Output=()> + Send>> {
    static SIGINT_HANDLER: OnceLock<Arc<SigintHandler<dyn Future<Output=()> + Send + 'static>>> = OnceLock::new();
    SIGINT_HANDLER.get_or_init(|| {
        let handler = SigintHandler {
            listeners: Arc::new(Mutex::new(Vec::new()))
        };
        Arc::new(handler)
    }).clone()
}

pub fn init_sigint() {
    thread::spawn(|| {
        let sigint_flag = Arc::new(AtomicBool::new(false));
        signal_hook::flag::register(signal_hook::consts::SIGINT, Arc::clone(&sigint_flag)).unwrap();
        while !sigint_flag.load(Ordering::Relaxed) {
            std::thread::sleep(std::time::Duration::from_secs(1));
        }
        let binding = global_sigint_handler();
        let mut listeners = binding.listeners.lock().unwrap();
        let mut futures = Vec::new();
        for listener in listeners.iter_mut() {
            futures.push(listener.as_mut());
        }
        let runtime = global_tokio_runtime();
        runtime.block_on(async move {
            for x in futures.iter_mut() {
                x.await;
            }
        });

        exit(0);
    });
/*    tokio::spawn(async {
        let sigint_flag = Arc::new(AtomicBool::new(false));
        signal_hook::flag::register(signal_hook::consts::SIGINT, Arc::clone(&sigint_flag)).unwrap();
        while !sigint_flag.load(Ordering::Relaxed) {
            tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        }
        let binding = global_sigint_handler();
        let mut listeners = binding.listeners.lock().unwrap();
        let mut futures = Vec::new();
        for listener in listeners.iter_mut() {
            futures.push(listener.as_mut());
        }
        let runtime = global_tokio_runtime();
        runtime.block_on(async move {
            for x in futures.iter_mut() {
                x.await;
            }
        });

        exit(0);
    });*/
}

impl SigintHandler<dyn Future<Output=()> + Send + 'static> {
    pub fn add_listener(&self, listener: Pin<Box<dyn Future<Output=()> + Send + 'static>>) {
        let mut listeners = self.listeners.lock().unwrap();
        listeners.push(listener);
    }
}