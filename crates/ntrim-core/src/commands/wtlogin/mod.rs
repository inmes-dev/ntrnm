mod tlv;
pub mod refresh_sig;
pub use wtlogin_request::WtloginRequest;

pub mod wtlogin_request {
    use std::collections::HashMap;
    use std::ops::Deref;
    use std::sync::Arc;
    use anyhow::Error;
    use bytes::{Buf, BufMut, Bytes, BytesMut};
    use chrono::DateTime;
    use log::{info, warn};
    use tokio::sync::oneshot::{Receiver, Sender};
    use ntrim_tools::bytes::{BytePacketBuilder, BytePacketReader, PacketFlag};
    use ntrim_tools::crypto::ecdh::ecdh_share_key;
    use ntrim_tools::crypto::qqtea::{qqtea_decrypt, qqtea_encrypt};
    use crate::client::packet::FromServiceMsg;
    use crate::client::packet::packet::{CommandType, UniPacket};
    use crate::client::qsecurity::QSecurity;
    use crate::client::trpc::TrpcClient;
    use crate::events::wtlogin_event::WtloginResponse;
    use crate::session::SsoSession;
    use crate::session::ticket::{SigType, Ticket, TicketManager};

    pub trait WtloginFactory<R: WtloginRequest> {
        type Params: ?Sized;

        fn build(trpc: Arc<TrpcClient>, params: Self::Params) -> Arc<WtloginBuilder<R>>;
    }

    pub struct WtloginBuilder<T: WtloginRequest> {
        pub trpc: Arc<TrpcClient>,
        pub command: String,
        pub command_type: CommandType,
        pub wt_command: u16,
        pub wt_sub_command: u16,
        pub request: T
    }

    impl<T: WtloginRequest> Deref for WtloginBuilder<T> {
        type Target = T;

        fn deref(&self) -> &Self::Target {
            &self.request
        }
    }

    impl<T: WtloginRequest + Sync + Send + 'static> WtloginBuilder<T>
    where T: WtloginRequest
    {
        async fn generate_body(&self, session: &SsoSession, seq: u32) -> Vec<u8> {
            let encrypt_body = self.request.generate_encrypt_body();
            let encrypt_key = self.request.get_encrypt_key(session);
            let encrypt_public_key = encrypt_key.1;
            let encrypt_share_key = encrypt_key.2;
            let tlv_body = self.request.generate_tlv_body(
                session, self.trpc.qsec.clone(),
                self.wt_sub_command, seq
            ).await;
            let encrypt_tlv_body = qqtea_encrypt(tlv_body.as_slice(), encrypt_share_key.as_slice());
            let mut buf = BytesMut::new();
            buf.put_u8(2);
            buf.put_u16(27 + encrypt_body.len() as u16 + 2 + encrypt_public_key.len() as u16 + 2 + encrypt_tlv_body.len() as u16);
            buf.put_u16(8001);
            buf.put_u16(self.wt_command);
            buf.put_u16(1);
            buf.put_u32(session.uin as u32);
            buf.put_u8(3);
            buf.put_u8(encrypt_key.0);
            buf.put_u8(0);
            buf.put_u32(2);
            buf.put_u32(0);
            buf.put_u32(0);
            buf.put_slice(encrypt_body.as_slice());
            buf.put_bytes_with_flags(encrypt_public_key.as_slice(), PacketFlag::I16Len);
            buf.put_slice(encrypt_tlv_body.as_slice());
            buf.put_u8(3);
            buf.to_vec()
        }

        pub async fn send(self: Arc<Self>) -> Receiver<WtloginResponse> {
            let (tx, rx) = tokio::sync::oneshot::channel();
            let trpc = &self.trpc;
            let session = trpc.session.read().await;
            let seq = session.next_seq();
            let body = self.generate_body(session.deref(), seq).await;
            std::mem::drop(session); // make sure to release the lock
            let request = self.clone();
            tokio::spawn(async move {
                let trpc = &request.trpc;
                let uni_packet = UniPacket::new(request.command_type, request.command.clone(), body);
                if let Some(rx) = trpc.send_uni_packet_with_seq(uni_packet, seq).await {
                    match rx.await {
                        Ok(msg) => request.handle_response(msg, tx).await,
                        Err(e) => if !tx.is_closed() {
                            tx.send(WtloginResponse::Fail(Error::msg(format!("Failed to recv wtlogin request: {}", e)))).unwrap();
                        }
                    }
                } else {
                    if !tx.is_closed() {
                        tx.send(WtloginResponse::Fail(Error::msg("Failed to send wtlogin request"))).unwrap();
                    }
                }
            });
            return rx;
        }

        async fn handle_response(&self, msg: FromServiceMsg, cb: Sender<WtloginResponse>) {
            let mut session = self.trpc.session.write().await;
            let mut reader = BytesMut::from(msg.wup_buffer.as_slice());
            reader.advance(1 + 2 + 2 + 2 + 2);
            // 02 (dis) xx xx (dis) 1f 41 (dis) 08 01 (dis) 00 01 (dis)
            let uin = reader.get_u32() as u64;
            // println(uin)
            reader.advance(2);
            // 00 00 (dis)
//            val subCommand = reader.readShort().toInt() // subCommand discardExact 2
//            println(subCommand)
            let result = reader.get_u8();
            // 235 协议版本过低
            //let teaKey = if (result == 180) manager.session.randomKey else key

            if result != 0 {
                if cb.is_closed() { return; }
                cb.send(
                    WtloginResponse::Fail(Error::msg(format!("Wtlogin failed, result: {}, user_id: {}", result, uin)))
                ).unwrap();
                return;
            }

            let key = match self.command_type {
                CommandType::ExchangeSt => ecdh_share_key().await.as_slice(),
                CommandType::ExchangeSig => session.wt_session_key.as_slice(),
                _ => panic!("Not supported wtlogin command: {:?}", self.command_type),
            };

            let mut tlv_body = vec![0u8; reader.remaining() - 1];
            reader.copy_to_slice(&mut tlv_body);
            let tlv_body = qqtea_decrypt(tlv_body.as_slice(), key).to_vec();
            let mut tlv_body = BytesMut::from(tlv_body.as_slice());
            //let wt_sub_command = tlv_body.get_u16();
            tlv_body.advance(3); // wt_sub_command 00
            let tlv_map = Self::parse_tlv(&mut tlv_body);
            if let Some(t119) = tlv_map.get(&0x119) {
                let decrypt_key = match self.command_type {
                    CommandType::ExchangeSt => md5::compute(session.get_session_key(self.command_type)).0.to_vec(),
                    CommandType::ExchangeSig => session.tgtgt_key.clone(),
                    _ => panic!("Not supported wtlogin command: {:?}", self.command_type),
                };
                let t119 = qqtea_decrypt(t119.as_ref(), decrypt_key.as_slice());
                let mut t119 = BytesMut::from(t119.as_slice());
                Self::parse_tlv(&mut t119).iter().for_each(|(k, v)| {
                    match *k {
                        0x103 => {
                            // web sig
                        }
                        0x106 => {
                            session.encrypt_a1 = v.to_vec();
                            info!("Refresh encrypt_a1 successfully!");
                        }
                        0x10a => {
                            let mut ticket = session.ticket_mut(SigType::A2).unwrap();
                            ticket.sig = Some(v.to_vec());
                        }
                        0x10c => {
                            session.tgtgt_key = v.to_vec();
                            info!("Refresh gt_key successfully!");
                        }
                        0x10d => {
                            let mut ticket = session.ticket_mut(SigType::A2).unwrap();
                            ticket.sig_key = v.to_vec();
                        }
                        0x10e => {
                            let mut ticket = session.ticket_mut(SigType::ST).unwrap();
                            ticket.sig_key = v.to_vec();
                        }
                        0x114 => {
                            let mut ticket = session.ticket_mut(SigType::ST).unwrap();
                            ticket.sig = Some(v.to_vec());
                        }
                        0x118 => {}
                        0x11a => {
                            let mut buf = BytesMut::from(v.as_ref());
                            let face = buf.get_u16();
                            let age = buf.get_u8();
                            let gender = buf.get_u8();
                            let nick_len = buf.get_u8() as usize;
                            let nick = buf.copy_to_bytes(nick_len);
                            let nick = String::from_utf8(nick.to_vec()).unwrap();
                            info!("refresh success, nick: {}", nick);
                        }
                        0x11d => {
                            /*
                             session.encryptAppid = readUInt().toLong()
                    userStInfo.downloadStKey = bsTicket(readBytes(16))
                    userStInfo.downloadSt = bsTicket(readBytes(readUShort().toInt()))
                             */
                        }
                        0x11f => {
                            /*
                            session.appPriChangeTime = readUInt().toLong()
                    session.appPri = readUInt().toLong()
                             */
                        }
                        0x120 => {
                            session.skey = String::from_utf8(v.to_vec()).unwrap();
                            info!("Refresh sKey successfully!");
                        }
                        0x130 => {}
                        0x133 => {
                            session.wt_session_ticket = v.to_vec();
                        }
                        0x134 => {
                            session.wt_session_key = v.to_vec();
                        }
                        0x136 => {}
                        0x138 => {
                            let current_time_sec = chrono::Utc::now().timestamp() as u64;
                            let mut v = BytesMut::from(v.as_ref());
                            let count = v.get_u32();
                            for _ in 0..count {
                                let ver = v.get_u16();
                                let time = v.get_u32();

                                let expire_time = time as u64 + current_time_sec;
                                let expire_time = DateTime::from_timestamp(expire_time as i64, 0).unwrap();
                                info!("t{:x} expired time：{:?}", ver, expire_time);

                                match ver {
                                    0x106 => {},
                                    0x10a => {}
                                    0x11c => {}
                                    0x102 => {}
                                    0x103 => {},
                                    0x120 => {},
                                    0x143 => {
                                        let mut ticket = session.ticket_mut(SigType::D2).unwrap();
                                        ticket.create_time = current_time_sec;
                                        ticket.expire_time = time;
                                    }
                                    0x164 => {}
                                    _ => warn!("Unknown tlv_t{:x}", ver)
                                }
                                v.advance(4);
                            }
                        }
                        0x143 => {
                            let mut ticket = session.ticket_mut(SigType::D2).unwrap();
                            ticket.sig = Some(v.to_vec());
                        }
                        0x163 => {}
                        0x16a => {
                            session.no_pic_sig = v.to_vec();
                            info!("Refresh no_pic_sig successfully!");
                        }
                        0x16d => {
                            session.pskey = String::from_utf8(v.to_vec()).unwrap();
                        }
                        0x203 => {
                            // da2
                        }
                        0x305 => {
                            let mut ticket = session.ticket_mut(SigType::D2).unwrap();
                            ticket.sig_key = v.to_vec();
                            info!("Refresh d2key successfully!");
                        }
                        0x322 => {
                            // device token
                        }
                        0x512 => {
                            // web key
                            //let mut buf = v.clone().as_mut();
                            //let size = buf.get_u16();
                            //for _ in 0..size {
                            //    let domain = buf.get_str_with_flags(PacketFlag::I16Len).unwrap();
                            //    let pskey = buf.get_str_with_flags(PacketFlag::I16Len).unwrap();
                            //    let p4token = buf.get_str_with_flags(PacketFlag::I16Len).unwrap();
                            //}
                        }
                        0x522 => {}
                        0x528 => {}
                        0x537 => {
                            let mut v = BytesMut::from(v.as_ref());
                            let version = v.get_u8();
                            let count = v.get_u8();
                            for _ in 0..count {
                                let uin = v.get_u64();
                                let ip = v.copy_to_bytes(4);
                                let time = v.get_u32();
                                let app_id = v.get_u32();
                            }
                        }
                        0x543 => {

                        }
                        0x550 => {}

                        _ => warn!("Unknown tlv_t{:x}", k)
                    }
                });
            }
            if !cb.is_closed() {
                cb.send(WtloginResponse::Success()).expect("Failed to send wtlogin response");
            }
        }

        fn parse_tlv(tlv_body: &mut BytesMut) -> HashMap<u16, Bytes> {
            let tlv_cnt = tlv_body.get_u16();
            (0..tlv_cnt).map(|_| {
                let tlv_type = tlv_body.get_u16();
                let tlv_len = tlv_body.get_u16();
                let tlv_value = tlv_body.copy_to_bytes(tlv_len as usize);
                (tlv_type, tlv_value)
            }).collect::<HashMap<u16, Bytes>>()
        }
    }

    pub trait WtloginRequest {
        fn get_encrypt_key(&self, session: &SsoSession) -> (u8, Vec<u8>, Vec<u8>);

        fn generate_encrypt_body(&self) -> Vec<u8>  { vec![] }

        async fn generate_tlv544(&self, session: &SsoSession, qsec: Arc<dyn QSecurity>) -> Vec<u8>;

        async fn generate_tlv_body(
            &self,
            session: &SsoSession,
            qsec: Arc<dyn QSecurity>,
            wt_command: u16,
            seq: u32
        ) -> Vec<u8>;
    }
}

