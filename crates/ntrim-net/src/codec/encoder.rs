use tokio::io::AsyncWriteExt;
use tokio::net::tcp::OwnedWriteHalf;
use crate::codec::packet::UniPacket;
use crate::error::CodecError;

#[derive(Debug)]
pub(crate) struct Encoder {
    tx: OwnedWriteHalf,
}

impl Encoder {
    pub(crate) fn new(tx: OwnedWriteHalf) -> Self {
        Self { tx }
    }

    pub async fn send_packet(&mut self, uni_packet: UniPacket) -> Result<(), CodecError> {

        Ok(())
    }
}

impl Drop for Encoder {
    fn drop(&mut self) {
        let _ = self.tx.shutdown();
    }
}