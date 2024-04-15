pub(crate) mod oneshot;
pub(crate) mod persistent;
pub(crate) mod handler;

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use crate::client::dispatcher::oneshot::OneshotHandler;
use crate::client::dispatcher::persistent::PersistentHandler;
use crate::client::packet::from_service_msg::FromServiceMsg;

pub(crate) struct TrpcDispatcher {
    pub(crate) persistent: Arc<Mutex<HashMap<String, PersistentHandler>>>,
    pub(crate) oneshot: Arc<Mutex<HashMap<u32, OneshotHandler>>>,
}

impl TrpcDispatcher {
    pub fn new() -> Self {
        Self {
            persistent: Arc::new(Mutex::new(HashMap::new())),
            oneshot: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub(crate) async fn dispatch(&self, msg: FromServiceMsg) {
        let cmd = msg.command.as_str();

    }
}