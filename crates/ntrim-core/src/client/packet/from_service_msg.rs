use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct FromServiceMsg {
    pub command: String,
    /// not with data length
    pub wup_buffer: Arc<Vec<u8>>,
    pub seq: i32
}

impl FromServiceMsg {
    pub fn new(
        command: String,
        wup_buffer: Vec<u8>,
        seq: i32
    ) -> Self {
        Self {
            command, wup_buffer: Arc::new(wup_buffer), seq
        }
    }
}