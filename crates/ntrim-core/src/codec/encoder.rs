use std::ops::Deref;
use bytes::{BufMut, BytesMut};
use once_cell::sync::Lazy;
use prost::Message;
use tokio_util::codec::Encoder;
use ntrim_tools::bytes::BytePacketBuilder;

use crate::client::Client;
use crate::codec::CodecError;
use crate::packet::to_service_msg::ToServiceMsg;
use crate::pb::qqsecurity::QqSecurity;
use crate::sesson::protocol::Protocol;
use crate::sesson::SsoSession;

pub(crate) static DEFAULT_TEA_KEY: Lazy<[u8; 16]> = Lazy::new(|| {
    [0u8; 16]
});


impl Encoder<ToServiceMsg> for Client {
    type Error = CodecError;

    fn encode(&mut self, to_service_msg: ToServiceMsg, dst: &mut BytesMut) -> Result<(), Self::Error> {
        let uni_packet = to_service_msg.uni_packet;
        let session = &self.session;

        dst.put_packet_with_i32_len(&mut |buf| {
            let is_online = session.is_online();
            let encrypted_flag = uni_packet.get_encrypted_flag();
            let sso_seq = to_service_msg.seq;

            generate_surrounding_packet(
                buf, is_online, encrypted_flag,
                sso_seq, &to_service_msg.first_token,
                session.uin.to_string().as_bytes()
            );

            let tea_key = session.get_session_key();

            generate_0a_packet_head(uni_packet.command.as_str(), sso_seq, &to_service_msg.second_token, session);

            (buf.len() + 4) as i32
        });

        Ok(())
    }
}

/// Generate outermost packet
#[inline]
fn generate_surrounding_packet(
    buf: &mut BytesMut,
    is_online: bool,
    encrypted_flag: u8,
    seq: u32,
    first_token: &Option<Box<[u8]>>,
    uin: &[u8]
) {
    if is_online {
        buf.put_u32(0xB);
    } else {
        buf.put_u32(0xA);
    }
    buf.put_u8(encrypted_flag);
    if encrypted_flag == 0x1 {
        buf.put_u32(seq);
    } else {
        if let Some(first_token) = first_token {
            buf.put_bytes_with_i32_len(first_token, first_token.len() + 4);
        } else {
            buf.put_u32(4); // empty token
        }
    }

    buf.put_bytes_with_i32_len(uin, uin.len() + 4);
}

#[inline]
fn generate_0a_packet_head(
    command: &str,
    seq: u32,
    second_token: &Option<Box<[u8]>>,
    session: &SsoSession
) -> Vec<u8> {
    let protocol = &session.protocol;
    let app_id = protocol.app_id;
    let mut buf = BytesMut::new();

    buf.put_u32(seq);
    buf.put_u32(app_id);
    buf.put_u32(app_id);
    buf.put_u32(16777216);
    buf.put_u32(0);
    if let Some(second_token) = second_token {
        buf.put_u32(256);
        buf.put_bytes_with_i32_len(second_token, second_token.len() + 4);
    } else {
        buf.put_u32(0);
        buf.put_u32(4);
    }

    let device = &session.device;
    buf.put_bytes_with_i32_len(command.as_bytes(), command.len() + 4);
    buf.put_bytes_with_i32_len(&session.msg_cookie, session.msg_cookie.len() + 4);
    buf.put_bytes_with_i32_len(device.android_id.as_bytes(), device.android_id.len() + 4);
    buf.put_bytes_with_i32_len(&session.ksid, session.ksid.len() + 4);
    buf.put_bytes_with_i16_len(&protocol.detail.as_bytes(), protocol.detail.len() + 2);

    let mut qq_sec = QqSecurity::default();
    qq_sec.flag = 1;


    let qq_sec = qq_sec.encode_to_vec();
    buf.put_bytes_with_i32_len(qq_sec.as_slice(), qq_sec.len() + 4);

    buf.to_vec()
}