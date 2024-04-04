use std::ops::Deref;
use bytes::{BufMut, BytesMut};
use once_cell::sync::Lazy;
use tokio_util::codec::Encoder;

use crate::client::Client;
use crate::codec::CodecError;
use crate::packet::to_service_msg::ToServiceMsg;

pub(crate) static DEFAULT_TEA_KEY: Lazy<[u8; 16]> = Lazy::new(|| {
    [0u8; 16]
});


impl Encoder<ToServiceMsg> for Client {
    type Error = CodecError;

    fn encode(&mut self, to_service_msg: ToServiceMsg, dst: &mut BytesMut) -> Result<(), Self::Error> {
        let uni_packet = to_service_msg.uni_packet;


/*        if to_service_msg.is_login && uni_packet.get_encrypted_flag() == 0x1 && client_tea_key.is_some() {
            tea_key = client_tea_key.unwrap().as_ref();
        };
        dst.put_packet_with_i32_len(&mut |buf| {
            buf.put_u8(uni_packet.get_encrypted_flag());
            if uni_packet.get_encrypted_flag() == 0x1 {
                buf.put_u32(to_service_msg.seq);
            } else {
                if let Some(first_token) = to_service_msg.first_token.as_ref() {
                    buf.put_u32((first_token.len() + 4) as u32);
                    buf.put_slice(&first_token);
                } else {
                    buf.put_u32(4); // empty token
                }
            }

            let uin = uni_packet.uin.as_str();
            buf.put_u32(uin.len() as u32 + 4);
            buf.put_slice(uin.as_bytes());



            (buf.len() + 4) as i32
        });*/

        Ok(())
    }
}

fn generator_head() {

}