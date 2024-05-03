use std::sync::Arc;
use std::sync::atomic::Ordering::SeqCst;
use bytes::{Buf, BufMut, BytesMut};
use log::{debug, error, info, warn};
use tokio::io::AsyncReadExt;
use tokio::net::tcp::OwnedReadHalf;
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::sync::RwLockReadGuard;
use ntrim_tools::bytes::{BytePacketBuilder, BytePacketReader, PacketFlag};
use ntrim_tools::crypto::qqtea::qqtea_decrypt;
use ntrim_tools::flate2::decompress_deflate;

pub use crate::client::codec::CodecError;
use crate::client::codec::encoder::DEFAULT_TEA_KEY;
use crate::client::packet::from_service_msg::FromServiceMsg;
use crate::client::packet::packet::CommandType::Service;
use crate::client::packet::to_service_msg::ToServiceMsg;
use crate::client::tcp::TcpStatus;
use crate::client::trpc::TrpcClient;
use crate::session::SsoSession;

pub(crate) trait TrpcDecoder {
    fn init(self: &Arc<Self>);
}

impl TrpcDecoder for TrpcClient {
    fn init(self: &Arc<Self>) {
        let reader = self.client.reader();
        let trpc = Arc::clone(self);
        tokio::spawn(async move {
            let mut reader = reader.lock().await;
            debug!("Decoder is started: {:?}", reader);
            let session = trpc.session.clone();
            loop {
                if !trpc.client.is_connected() {
                    warn!("Connection is not connected: {:?}, decoder is canceled", trpc.client);
                    return;
                }
                if let Err(e) = reader.readable().await {
                    warn!("Tcp stream is not readable: {:?}", e);
                    let mut status = TcpStatus::from_bits(trpc.client.status.load(SeqCst)).unwrap();
                    status.set(TcpStatus::Lost, true);
                    trpc.client.status.store(status.bits(), SeqCst);
                    break;
                }
                let packet_size = match reader.read_u32().await {
                    Ok(0) => {
                        warn!("Connection closed by peer: {:?}", trpc.client);
                        let mut status = TcpStatus::from_bits(trpc.client.status.load(SeqCst)).unwrap();
                        status.set(TcpStatus::Lost, true);
                        trpc.client.status.store(status.bits(), SeqCst);
                        break;
                    }
                    Ok(size) => size,
                    Err(e) => {
                        let mut status = TcpStatus::from_bits(trpc.client.status.load(SeqCst)).unwrap();
                        status.set(TcpStatus::Lost, true);
                        trpc.client.status.store(status.bits(), SeqCst);
                        warn!("Failed to read packet size: {}", e);
                        break;
                    }
                } ;
                let mut buffer = vec![0u8; (packet_size - 4) as usize];
                // 读满data
                match reader.read_exact(&mut buffer).await {
                    Ok(n) => {
                        if n != buffer.len() {
                            warn!("Failed to read packet data: read {} bytes, expect {}", n, buffer.len());
                            break;
                        }
                        debug!("Read packet buf: {}", n);
                    }
                    Err(e) => {
                        warn!("Failed to read packet data: {}", e);
                        break;
                    }
                }
                let mut src = buffer.as_slice();
                let head_flag = src.get_u32();
                if head_flag == 0x01335239 {
                    debug!("Recv hello from server: MSF");
                    continue;
                }
                let encrypted_flag = src.get_u8();
                let session = session.read().await;
                debug!("Fetch session rwlock: {:?}", session);

                let tea_key = match encrypted_flag {
                    1 => session.get_session_key(Service),
                    _ => &*DEFAULT_TEA_KEY
                };
                if tea_key.len() != 16 {
                    warn!("Failed to get session key or tea key is invalid!");
                    continue;
                }

                src.get_i8(); // skip 0x0

                let user_id = src.get_str_with_flags(PacketFlag::I32Len | PacketFlag::ExtraLen).unwrap();

                // read rest of the packet (no length)
                let mut data = vec![0u8; src.remaining()];
                src.copy_to_slice(&mut data);
                let data = qqtea_decrypt(&data, tea_key);
                let mut data = BytesMut::from(data.as_slice());

                let (seq, cmd, compression) = parse_head(&mut data);

                info!("Recv packet from user_id: {}, cmd: {}, seq: {}", user_id, cmd, seq);

                let mut body = vec![0u8; (data.get_u32() - 4) as usize];
                data.copy_to_slice(&mut body);
                //info!("Recv packet body: {:?}", hex::encode(&body));
                let body = match compression {
                    0 => body,
                    4 => body,
                    1 => decompress_deflate(&body),
                    _ => body
                };

                let from_service_msg = FromServiceMsg::new(cmd, body, seq);
                let dispatcher = Arc::clone(&trpc.dispatcher);
                tokio::spawn(async move {
                    dispatcher.dispatch(from_service_msg).await;
                });
            }
            error!("Decoder is canceled: {:?}", trpc.client)
        });
    }
}

#[inline]
fn parse_head(data: &mut BytesMut) -> (i32, String, u32) {
    let head_length = data.get_u32() - 4;
    let mut head_data = data.split_to(head_length as usize);
    let seq = head_data.get_i32();
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