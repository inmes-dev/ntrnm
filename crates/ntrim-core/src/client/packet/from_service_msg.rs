use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct FromServiceMsg {
    pub command: String,
    /// without data length
    pub wup_buffer: Vec<u8>,
    pub seq: i32
}

impl FromServiceMsg {
    pub fn new(
        command: String,
        wup_buffer: Vec<u8>,
        seq: i32
    ) -> Self {
        Self {
            command, wup_buffer, seq
        }
    }
}