use std::ops::Deref;
use bytes::{BufMut, BytesMut};
use once_cell::sync::Lazy;
use prost::Message;
use tokio_util::codec::Encoder;
use ntrim_tools::bytes::BytePacketBuilder;
use ntrim_tools::crypto::qqtea::qqtea_encrypt;

use crate::client::Client;
use crate::codec::CodecError;
use crate::codec::qqsecurity::{QSecurityResult};
use crate::packet::to_service_msg::ToServiceMsg;
use crate::pb::qqsecurity::{QqSecurity, SsoMapEntry, SsoSecureInfo};
use crate::sesson::SsoSession;

pub(crate) static DEFAULT_TEA_KEY: Lazy<[u8; 16]> = Lazy::new(|| {
    [0u8; 16]
});

impl Encoder<ToServiceMsg> for Client {
    type Error = CodecError;

    fn encode(&mut self, to_service_msg: ToServiceMsg, dst: &mut BytesMut) -> Result<(), Self::Error> {
        let uni_packet = to_service_msg.uni_packet;
        let session = &self.session;
        let device = &session.device;
        let sso_seq = to_service_msg.seq;

        let qq_sec = generate_qqsecurity_head(
            (session.uin, session.uid.as_str()), &device.qimei, to_service_msg.sec_info
        );

        dst.put_packet_with_i32_len(&mut |buf| {
            let is_online = session.is_online();
            let encrypted_flag = uni_packet.get_encrypted_flag();

            generate_surrounding_packet(
                buf, is_online, encrypted_flag,
                sso_seq, &to_service_msg.first_token,
                session.uin.to_string().as_bytes()
            );

            let tea_key = session.get_session_key();

            let mut data = BytesMut::new();
            let head_body = match session.is_online() {
                true => generate_0a_packet_head(uni_packet.command.as_str(), sso_seq, &to_service_msg.second_token, session, &qq_sec),
                false => generate_0b_packet_head(uni_packet.command.as_str(), session, &qq_sec)
            };
            data.put_bytes_with_i32_len(head_body.as_slice(), head_body.len() + 4);
            let wup_buffer = uni_packet.to_wup_buffer();
            data.put_bytes_with_i32_len(wup_buffer.as_slice(), wup_buffer.len() + 4);

            let data = qqtea_encrypt(&data, tea_key);
            buf.put(data.as_slice());

            (buf.len() + 4) as i32
        });

        Ok(())
    }
}

#[inline]
fn generate_qqsecurity_head(
    account: (u64, &str),
    qimei: &str,
    qsec_info: Option<QSecurityResult>
) -> Vec<u8> {
    use rand::Rng;
    use std::fmt::Write;

    pub fn generate_trace() -> String {
        let hex = "0123456789ABCDEF".chars().collect::<Vec<char>>();
        let mut rng = rand::thread_rng();
        let mut string = String::with_capacity(55);

        write!(&mut string, "{:02X}", 0).unwrap();
        string.push('-');

        for _ in 0..32 {
            let index = rng.gen_range(0..hex.len());
            string.push(hex[index]);
        }
        string.push('-');

        for _ in 0..16 {
            let index = rng.gen_range(0..hex.len());
            string.push(hex[index]);
        }
        string.push('-');

        write!(&mut string, "{:02X}", 1).unwrap();

        string
    }

    let mut qq_sec = QqSecurity::default();

    if let Some(result) = qsec_info {
/*        let wup_buffer = packet.to_wup_buffer();
        let result = qsec.sign(account.0.to_string(), packet.command.clone(), wup_buffer, seq);
        let mut sec_info = SsoSecureInfo::default();
        sec_info.device_token.put_slice(result.token.deref());
        sec_info.sec_sig.put_slice(result.sign.deref());
        sec_info.extra.put_slice(result.extra.deref());
        qq_sec.sec_info = Some(sec_info);*/
        let mut sec_info = SsoSecureInfo::default();
        sec_info.device_token.put_slice(result.token.deref());
        sec_info.sec_sig.put_slice(result.sign.deref());
        sec_info.extra.put_slice(result.extra.deref());
        qq_sec.sec_info = Some(sec_info);
    }
    qq_sec.flag = 1;
    qq_sec.locale_id = 2052;
    qq_sec.qimei = qimei.to_string();
    qq_sec.trace_parent = generate_trace();
    qq_sec.uid = account.1.to_string();
    qq_sec.network_type = 0;
    qq_sec.unknown = 1;
    qq_sec.ip_stack_type = 1;
    qq_sec.message_type = 34;

    // 获取当前时间戳
    let timestamp = chrono::Utc::now().timestamp().to_string();
    let entry = SsoMapEntry {
        key: "client_conn_seq".to_string(),
        value: timestamp.into_bytes(),
    };

    qq_sec.nt_core_version = 100;
    qq_sec.sso_ip_origin = 28;

    qq_sec.trans_info.push(entry);

    qq_sec.encode_to_vec()
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
fn generate_0b_packet_head(
    command: &str,
    session: &SsoSession,
    qq_sec: &Vec<u8>
) -> Vec<u8> {
    let mut buf = BytesMut::new();
    buf.put_bytes_with_i32_len(command.as_bytes(), command.len() + 4);
    buf.put_bytes_with_i32_len(&session.msg_cookie, session.msg_cookie.len() + 4);
    buf.put_bytes_with_i32_len(qq_sec.as_slice(), qq_sec.len() + 4);

    buf.to_vec()
}

#[inline]
fn generate_0a_packet_head(
    command: &str,
    seq: u32,
    second_token: &Option<Box<[u8]>>,
    session: &SsoSession,
    qq_sec: &Vec<u8>
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
    buf.put_bytes_with_i32_len(qq_sec.as_slice(), qq_sec.len() + 4);

    buf.to_vec()
}