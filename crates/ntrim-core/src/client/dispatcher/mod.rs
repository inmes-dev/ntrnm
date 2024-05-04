use std::collections::HashMap;
use std::sync::Arc;
use log::{debug, error, info};
use tokio::sync::{mpsc, Mutex, oneshot};
use crate::client::packet::from_service_msg::FromServiceMsg;

#[derive(Debug)]
pub(crate) struct TrpcDispatcher {
    pub(crate) persistent: Arc<Mutex<HashMap<String, mpsc::Sender<FromServiceMsg>>>>,
    pub(crate) oneshot: Arc<Mutex<HashMap<u32, oneshot::Sender<FromServiceMsg>>>>,
}

impl TrpcDispatcher {
    pub fn new() -> Self {
        Self {
            persistent: Arc::new(Mutex::new(HashMap::new())),
            oneshot: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn clear(&self) {
        let mut persistent = self.persistent.lock().await;
        persistent.clear();
        let mut oneshot = self.oneshot.lock().await;
        oneshot.clear();
    }

    pub async fn register_persistent(&self, cmd: String, sender: mpsc::Sender<FromServiceMsg>) {
        let mut persistent = self.persistent.lock().await;
        debug!("Registering persistent, cmd: {}", cmd);
        persistent.insert(cmd, sender);
    }

    pub async fn register_multiple_persistent(&self, cmd: Vec<String>, sender: mpsc::Sender<FromServiceMsg>) {
        let mut persistent = self.persistent.lock().await;
        debug!("Registering persistent, cmd: {:?}", cmd);
        cmd.iter().for_each(|c| {
            persistent.insert(c.clone(), sender.clone());
        });
    }

    pub async fn register_oneshot(&self, seq: u32, sender: oneshot::Sender<FromServiceMsg>) {
        let mut oneshot = self.oneshot.lock().await;
        debug!("Registering oneshot, seq: {}", seq);
        oneshot.insert(seq, sender);
    }

    pub(crate) async fn dispatch(self: Arc<Self>, msg: FromServiceMsg) {
        let cmd = msg.command.clone();
        let seq = msg.seq;

        debug!("Dispatching packet, cmd: {}, seq: {}", cmd, seq);

        if seq <= 0 {
            let persistent = self.persistent.lock().await;
            if let Some(sender) = persistent.get(&cmd) {
                if let Err(e) = sender.send(msg).await {
                    error!("Failed to send message to persistent map, dispatcher: {:?}, cmd: {}, err: {:?}", self, cmd, e);
                }
                return;
            }
        } else {
            let seq = seq as u32;
            let mut oneshot = self.oneshot.lock().await;
            if let Some(sender) = oneshot.remove(&seq) {
                if let Err(msg) = sender.send(msg) {
                    error!("Failed to send message to oneshot map, dispatcher: {:?}, seq: {}, msg: {:?}", self, seq, msg);
                }
                return;
            }

            error!("Failed to dispatch packet, seq: {}, cmd: {}", seq, cmd);
        }
    }
}