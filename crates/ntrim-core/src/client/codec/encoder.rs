use std::ops::Deref;
use std::sync::Arc;
use bytes::{BufMut, BytesMut};
use log::{debug, error, info, warn};
use once_cell::sync::Lazy;
use prost::Message;
use tokio::sync::mpsc::Receiver;
use tokio::sync::RwLockReadGuard;
use ntrim_tools::bytes::{BytePacketBuilder, PacketFlag};
use ntrim_tools::crypto::qqtea::qqtea_encrypt;

use crate::client::codec::CodecError;
use crate::client::packet::packet::UniPacket;
use crate::client::packet::to_service_msg::ToServiceMsg;
use crate::client::qsecurity::QSecurityResult;
use crate::client::trpc::{ClientError, TrpcClient};
use crate::pb::qqsecurity::{QqSecurity, SsoMapEntry, SsoSecureInfo};
use crate::session::SsoSession;

pub(crate) static DEFAULT_TEA_KEY: Lazy<[u8; 16]> = Lazy::new(|| {
    [0u8; 16]
});

pub(crate) trait TrpcEncoder {
    fn init(self: &Arc<Self>, rx: Receiver<ToServiceMsg>);
}

impl TrpcEncoder for TrpcClient {
    fn init(self: &Arc<Self>, mut rx: Receiver<ToServiceMsg>) {
        let mut trpc = Arc::clone(self);
        tokio::spawn(async move {
            let session = trpc.session.clone();
            loop {
                let packet = match rx.recv().await {
                    Some(packet) => packet,
                    None => {
                        debug!("Trpc-sender channel closed");
                        break;
                    }
                };
                let session = session.read().await;
                debug!("Fetch session rwlock: {:?}", session);
                let mut buf = BytesMut::new();
                if let Err(e) = encode(&session, packet, &mut buf) {
                    error!("Failed to encode packet: {:?}", e);
                    continue;
                }
                debug!("Sending packet to server, size: {}", buf.len());
                //info!("Packet: {:?}", hex::encode(buf));
                trpc.client.write_data(buf).await.unwrap_or_else(|e| {
                    error!("Failed to write data to server: {:?}", e);
                });
            }
        });
    }
}

fn encode(session: &RwLockReadGuard<SsoSession>, msg: ToServiceMsg, dst: &mut BytesMut) -> Result<(), CodecError> {
    let uni_packet = msg.uni_packet;
    let device = &session.device;
    let sso_seq = msg.seq;

    let qq_sec = generate_qqsecurity_head(
        (session.uin, session.uid.as_str()), &device.qimei, msg.sec_info
    );

    let tea_key = session.get_session_key(uni_packet.command_type);
    if tea_key.len() != 16 {
        return Err(CodecError::InvalidTeaKey);
    }

    debug!("Tea key: {:?}", hex::encode(tea_key));

    dst.put_packet_with_flags(&mut |buf| {
        let encrypted_flag = uni_packet.get_encrypted_flag();
        let head_flag = uni_packet.get_head_flag();

        generate_surrounding_packet(
            buf,
            head_flag,
            encrypted_flag,
            sso_seq,
            &msg.first_token,
            session.uin.to_string().as_bytes()
        );

        let mut data = BytesMut::new();
        let head_body = if head_flag == 0xA {
            generate_0a_packet_head(uni_packet.command.as_str(), sso_seq, &msg.second_token, session, &qq_sec)
        } else {
            generate_0b_packet_head(uni_packet.command.as_str(), session, &qq_sec)
        };
        data.put_bytes_with_flags(head_body.as_slice(), PacketFlag::I32Len | PacketFlag::ExtraLen);
        let wup_buffer = uni_packet.to_wup_buffer();
        //info!("Wup buffer: {:?}", hex::encode(wup_buffer.as_slice()));
        data.put_bytes_with_flags(wup_buffer.as_slice(), PacketFlag::I32Len | PacketFlag::ExtraLen);

        let data = qqtea_encrypt(&data, tea_key);
        buf.put(data.as_slice());
    }, PacketFlag::I32Len | PacketFlag::ExtraLen);

    Ok(())
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
        let hex = "0123456789abcdef".chars().collect::<Vec<char>>();
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

        write!(&mut string, "{:02X}", 0).unwrap();

        string
    } // 00-05b4b2f95cc99439d5ee97c2b61f68be-1a08105332e004e2-00

    let mut qq_sec = QqSecurity::default();

    if let Some(result) = qsec_info {
        let mut sec_info = SsoSecureInfo::default();
        sec_info.device_token.put_slice(result.token.deref());
        sec_info.sec_sig.put_slice(result.sign.deref());
        sec_info.extra.put_slice(result.extra.deref());
        qq_sec.sec_info = Some(sec_info);
    }
    qq_sec.flag = 1;
    qq_sec.locale_id = 2052;
    qq_sec.qimei = qimei.to_string(); //0902564ff81969dfb3596f2010001f91760e
    qq_sec.newconn_flag = 0;
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
    qq_sec.trans_info.push(entry);

    qq_sec.nt_core_version = 100;
    qq_sec.sso_ip_origin = 3;

    qq_sec.encode_to_vec()
}

/// Generate outermost packet
#[inline]
fn generate_surrounding_packet(
    buf: &mut BytesMut,
    head_flag: u32,
    encrypted_flag: u8,
    seq: u32,
    first_token: &Option<Box<Vec<u8>>>,
    uin: &[u8]
) {
    buf.put_u32(head_flag);
    buf.put_u8(encrypted_flag);
    if head_flag == 0xB {
        buf.put_u32(seq);
    } else {
        if let Some(first_token) = first_token {
            buf.put_bytes_with_flags(first_token, PacketFlag::I32Len | PacketFlag::ExtraLen);
        } else {
            buf.put_u32(4); // empty token
        }
    }
    buf.put_i8(0); // split

    buf.put_bytes_with_flags(uin, PacketFlag::I32Len | PacketFlag::ExtraLen);
}

#[inline]
fn generate_0b_packet_head(
    command: &str,
    session: &SsoSession,
    qq_sec: &Vec<u8>
) -> Vec<u8> {
    let mut buf = BytesMut::new();
    buf.put_bytes_with_flags(command.as_bytes(), PacketFlag::I32Len | PacketFlag::ExtraLen);
    buf.put_bytes_with_flags(&session.msg_cookie, PacketFlag::I32Len | PacketFlag::ExtraLen);
    buf.put_bytes_with_flags(qq_sec.as_slice(), PacketFlag::I32Len | PacketFlag::ExtraLen);

    buf.to_vec()
}

#[inline]
fn generate_0a_packet_head(
    command: &str,
    seq: u32,
    second_token: &Option<Box<Vec<u8>>>,
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
        buf.put_bytes_with_flags(second_token, PacketFlag::I32Len | PacketFlag::ExtraLen);
    } else {
        buf.put_u32(0);
        buf.put_u32(4);
    }

    let device = &session.device;
    buf.put_bytes_with_flags(command.as_bytes(), PacketFlag::I32Len | PacketFlag::ExtraLen);
    buf.put_i32(4);
    //buf.put_bytes_with_i32_len(&session.msg_cookie, session.msg_cookie.len() + 4);
    buf.put_bytes_with_flags(device.android_id.as_bytes(), PacketFlag::I32Len | PacketFlag::ExtraLen);
    buf.put_bytes_with_flags(&session.ksid, PacketFlag::I32Len | PacketFlag::ExtraLen);
    buf.put_bytes_with_flags(&protocol.detail.as_bytes(), PacketFlag::I16Len | PacketFlag::ExtraLen);
    buf.put_bytes_with_flags(qq_sec.as_slice(), PacketFlag::I32Len | PacketFlag::ExtraLen);

    buf.to_vec()
}