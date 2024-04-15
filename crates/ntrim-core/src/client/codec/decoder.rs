use std::sync::Arc;
use bytes::{Buf, BufMut, BytesMut};
use log::debug;
use tokio::io::AsyncReadExt;
use tokio::net::tcp::OwnedReadHalf;
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::sync::RwLockReadGuard;
use ntrim_tools::crypto::qqtea::qqtea_decrypt;
use ntrim_tools::flate2::decompress_deflate;

pub use crate::client::codec::CodecError;
use crate::client::codec::encoder::DEFAULT_TEA_KEY;
use crate::client::packet::from_service_msg::FromServiceMsg;
use crate::client::packet::packet::CommandType::Service;
use crate::client::packet::to_service_msg::ToServiceMsg;
use crate::client::trpc::TrpcClient;
use crate::sesson::SsoSession;

pub(crate) trait TrpcDecoder {
    fn init(self: &Arc<Self>);
}

impl TrpcDecoder for TrpcClient {
    fn init(self: &Arc<Self>) {
        let reader = self.client.get_reader();
        let trpc = Arc::clone(self);
        tokio::spawn(async move {
            let mut reader = reader.lock().await;
            loop {
                if !trpc.client.is_connected() {
                    return;
                }
                let packet_size = match reader.read_u32().await {
                    Ok(size) => size,
                    Err(e) => {
                        debug!("Failed to read packet size: {}", e);
                        break;
                    }
                };
                let mut data = vec![0u8; packet_size as usize];
                match reader.read_exact(&mut data).await {
                    Ok(_) => {}
                    Err(e) => {
                        debug!("Failed to read packet data: {}", e);
                        break;
                    }
                }
                let mut src = BytesMut::from(data.as_slice());
                let head_flag = src.get_u32();
                let encrypted_flag = src.get_u8();
                let session = trpc.session.read().await;

                let tea_key = match encrypted_flag {
                    1 => session.get_session_key(Service),
                    _ => &*DEFAULT_TEA_KEY
                };
                if tea_key.len() != 16 {
                    debug!("Failed to get session key or tea key is invalid!");
                    continue;
                }

                src.advance(1); // skip 0x0

                let mut user_id = vec![0u8; (src.get_u32() - 4) as usize];
                src.copy_to_slice(&mut user_id);
                let user_id: String = String::from_utf8(user_id).unwrap();

                // read rest of the packet (no length)
                let mut data = vec![0u8; src.remaining()];
                src.copy_to_slice(&mut data);
                let data = qqtea_decrypt(&data, tea_key);
                let mut data = BytesMut::from(data.as_slice());
                let (seq, cmd, compression) = parse_head(&mut data);

                debug!("Recv packet from user_id: {}, cmd: {}, seq: {}", user_id, cmd, seq);

                let mut body = vec![0u8; (data.get_u32() - 4) as usize];
                data.copy_to_slice(&mut body);
                let body = match compression {
                    0 => body,
                    4 => body,
                    1 => decompress_deflate(&body),
                    _ => body
                };
                let from_service_msg = FromServiceMsg::new(cmd, body, seq);
                let dispenser = Arc::clone(&trpc.dispenser);
                tokio::spawn(async move {
                    dispenser.dispatch(from_service_msg).await;
                });
            }
        });
    }
}

#[inline]
fn parse_head(data: &mut BytesMut) -> (u32, String, u32) {
    let head_length = data.get_u32();
    let mut head_data = data.split_to(head_length as usize);
    let seq = head_data.get_u32();
    head_data.advance(4); // skip repeated 0
    let unknown_token_len = (head_data.get_u32() - 4) as usize;
    head_data.advance(unknown_token_len); // skip unknown tk
    let mut cmd = vec![0u8; (head_data.get_u32() - 4) as usize];
    head_data.copy_to_slice(&mut cmd);
    let cmd: String = String::from_utf8(cmd).unwrap();
    head_data.advance(4 + 4); // skip session id
    let compression = head_data.get_u32();
    (seq, cmd, compression)
}