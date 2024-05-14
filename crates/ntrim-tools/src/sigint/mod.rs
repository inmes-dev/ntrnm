use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::sync::atomic::AtomicBool;
use std::{future, thread};
use std::process::exit;
use log::info;

use once_cell::sync::Lazy;
use tokio::runtime;

pub struct SigintHandler<F>
    where
        F: Future + Send + ?Sized + 'static,
        F::Output: 'static
{
    listeners: Arc<Mutex<Vec<Pin<Box<F>>>>>
}

pub static SIGINT_HANDLER: Lazy<Arc<SigintHandler<dyn Future<Output=()> + Send + 'static>>> = Lazy::new(|| {
    let handler = SigintHandler {
        listeners: Arc::new(Mutex::new(Vec::new()))
    };
    Arc::new(handler)
});

pub fn init_sigint() {
    thread::spawn(|| {
        let sigint_flag = Arc::new(AtomicBool::new(false));
        signal_hook::flag::register(signal_hook::consts::SIGINT, Arc::clone(&sigint_flag)).unwrap();
        while !sigint_flag.load(std::sync::atomic::Ordering::Relaxed) {
            std::thread::sleep(std::time::Duration::from_secs(1));
        }
        let mut listeners = SIGINT_HANDLER.listeners.lock().unwrap();
        let mut futures = Vec::new();
        for listener in listeners.iter_mut() {
            futures.push(listener.as_mut());
        }
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        runtime.block_on(async move {
            for x in futures.iter_mut() {
                x.await;
            }
        });

        exit(0);
    });
}

impl SigintHandler<dyn Future<Output=()> + Send + 'static> {
    pub fn add_listener(&self, listener: Pin<Box<dyn Future<Output=()> + Send + 'static>>) {
        let mut listeners = self.listeners.lock().unwrap();
        listeners.push(listener);
    }
}