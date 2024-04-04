use bytes::BytesMut;
use tokio::net::tcp::{OwnedReadHalf};
use tokio_util::codec::Decoder;
use crate::client::Client;
use crate::codec::CodecError;
use crate::packet::from_service_msg::FromServiceMsg;

impl Decoder for Client {
    type Item = FromServiceMsg;

    type Error = CodecError;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        todo!()
    }
}