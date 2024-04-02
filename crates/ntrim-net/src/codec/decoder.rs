use tokio::net::tcp::{OwnedReadHalf};
use crate::error::CodecError;

#[derive(Debug)]
pub(crate) struct Decoder {
    rx: OwnedReadHalf,
}

impl Decoder {
    pub(crate) fn new(rx: OwnedReadHalf) -> Self {
        Self { rx }
    }

    pub async fn recv_packet(&mut self) -> Result<(), CodecError> {

        Ok(())
    }
}

impl Drop for Decoder {
    fn drop(&mut self) {
        // OwnedReadHalf 不需要 shutdown
        // let _ = self.rx.shutdown();
    }
}