use std::fmt::Write;
use std::future::Future;
use std::sync::{Arc, Mutex};
use anyhow::Error;
use bytes::{BufMut, BytesMut};
use log::{debug, error, info, warn};
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::sync::{oneshot, RwLock};
use tokio::time::Duration;
use ntrim_tools::tokiort;
use crate::client::codec::decoder::TrpcDecoder;
use crate::client::codec::encoder::TrpcEncoder;
use crate::client::dispatcher::TrpcDispatcher;
use crate::client::packet::from_service_msg::FromServiceMsg;
use crate::client::packet::packet::CommandType::{*};
use crate::client::tcp::{TcpStatus, TcpClient};
use crate::client::packet::packet::UniPacket;
use crate::client::packet::to_service_msg::ToServiceMsg;
use crate::client::qsecurity::QSecurity;
use crate::session::SsoSession;
use crate::session::ticket::{SigType, TicketManager};
pub use crate::client::tcp::ClientError;

pub struct TrpcClient {
    pub(crate) client: Arc<RwLock<TcpClient>>,
    pub session: Arc<RwLock<SsoSession>>,
    pub qsec: Arc<dyn QSecurity>,
    pub sender: Arc<Sender<ToServiceMsg>>,
    pub dispatcher: Arc<TrpcDispatcher>
}

impl TrpcClient {
    pub async fn new(
        session: SsoSession,
        qsec_mod: Arc<dyn QSecurity>
    ) -> Result<Arc<Self>, ClientError> {
        let is_ipv6 = match option_env!("IS_NT_IPV6") {
            None => false,
            Some(value) => value == "1",
        };
        let (tx, rx) = tokio::sync::mpsc::channel(match option_env!("NT_SEND_QUEUE_SIZE") {
            None => 32,
            Some(value) => value.parse::<usize>().unwrap_or(32),
        });
        let trpc = Arc::new(Self {
            client: Arc::new(RwLock::new(if !is_ipv6 {
                TcpClient::new_ipv4_client()
            } else {
                TcpClient::new_ipv6_client()
            })),
            qsec: qsec_mod,
            session: Arc::new(RwLock::new(session)),
            sender: Arc::new(tx),
            dispatcher: Arc::new(TrpcDispatcher::new()),
        });
        trpc.repeat_ping_sign_server();
        trpc.try_connect().await?;
        TrpcEncoder::init(&trpc, rx);
        TrpcDecoder::init(&trpc).await;
        Ok(trpc)
    }

    pub async fn is_connected(self: &Arc<Self>) -> bool {
        let client = self.client.read().await;
        return client.is_connected();
    }

    pub async fn is_lost(self: &Arc<Self>) -> bool {
        let client = self.client.read().await;
        return client.is_lost();
    }

    pub async fn try_connect(self: &Arc<Self>) -> Result<(), ClientError> {
        let mut client = self.client.write().await;
        client.connect().await?;

        let mut buf = BytesMut::new();
        buf.put_u32(21);
        buf.put_u32(0x01335239);
        buf.put_u32(0x00000000);
        buf.put_u8(0x4);
        buf.write_str("MSF").unwrap();
        buf.put_u8(0x5);
        buf.put_u32(0x00000000);
        client.write_data(buf).await.unwrap();

        Ok(())
    }

    pub fn repeat_ping_sign_server(self: &Arc<Self>) {
        let trpc = Arc::clone(self);
        tokio::spawn(async move {
            let qsec = Arc::clone(&trpc.qsec);
            let mut count = 0;
            while trpc.is_connected().await {
                let status = qsec.ping().await;
                count += 1;
                if count == 50 {
                    info!("Pinging sign server, status: {}", status);
                    count = 0;
                }
                tokio::time::sleep(Duration::from_secs(15)).await;
            }
        });
    }

    pub async fn send_uni_packet(self: &Arc<TrpcClient>, uni_packet: UniPacket) -> (u32, Option<oneshot::Receiver<FromServiceMsg>>) {
        let session = self.session.clone();
        let session = session.read().await;
        let seq = session.next_seq();
        return (seq, self.send_uni_packet_with_seq(uni_packet, seq).await);
    }

    pub async fn send_uni_packet_with_seq(self: &Arc<TrpcClient>, uni_packet: UniPacket, seq: u32) -> Option<oneshot::Receiver<FromServiceMsg>> {
        if !self.is_connected().await || self.is_lost().await {
            return None;
        }

        let (tx, rx) = oneshot::channel();
        let session = self.session.clone();
        let session = session.read().await;
        debug!("Fetch session rwlock: {:?}", session);

        let cmd = uni_packet.command.clone();

        let sec_info = if self.qsec.is_whitelist_command(cmd.as_str()).await {
            Some(self.qsec.sign(
                session.uin.to_string(),
                uni_packet.command.clone(),
                uni_packet.wup_buffer.clone(),
                seq
            ).await)
        } else {
            None
        };

        let mut msg = ToServiceMsg::new(uni_packet, seq);
        if let Some(sec_info) = sec_info {
            if sec_info.sign.is_empty() {
                error!("Failed to sign packet, seq: {}, cmd: {}", seq, cmd);
                return None;
            } else {
                msg.sec_info = Option::from(sec_info);
            }
        }

        info!("Send packet, cmd: {}, seq: {}", cmd, seq);

        match msg.uni_packet.command_type {
            Register => {
                let d2 = session.ticket(SigType::D2).unwrap();
                let d2 = d2.sig.clone().unwrap();
                msg.first_token = Some(Box::new(d2));
                let tgt = session.ticket(SigType::A2).unwrap();
                let tgt = tgt.sig.clone().unwrap();
                msg.second_token = Some(Box::new(tgt));
            }
            Service => {
                // nothing
            }
            ExchangeSt => {
                // nothing
            }
            ExchangeSig => {
                // nothing
            }
            _ => {
                error!("Invalid command type: {:?}", msg.uni_packet.command_type);
            }
        }
        self.dispatcher.register_oneshot(seq, tx).await;
        if let Err(e) = self.sender.send(msg).await {
            error!("Failed to send packet account: {:?} ,err: {}", session.uin, e);
            return None;
        }
        return Some(rx);
    }

    pub async fn unregister_oneshot(self: &Arc<TrpcClient>, seq: u32) {
        self.dispatcher.unregister_oneshot(seq).await;
    }

    pub async fn register_persistent(self: &Arc<TrpcClient>, cmd: String, sender: Sender<FromServiceMsg>) {
        self.dispatcher.register_persistent(cmd, sender).await;
    }

    pub async fn register_multiple_persistent(self: &Arc<TrpcClient>, cmd: Vec<String>, sender: Sender<FromServiceMsg>) {
        self.dispatcher.register_multiple_persistent(cmd, sender).await;
    }

    pub async fn set_lost(self: &Arc<Self>) {
        let mut client = self.client.write().await;
        client.set_lost().await;
        warn!("Trpc connection lost");
        self.dispatcher.clear_oneshot().await;
    }

    pub async fn disconnect(self: &Arc<Self>) {
        let mut client = self.client.write().await;
        client.disconnect().await;
        self.dispatcher.clear().await;
        self.sender.closed().await;
    }
}
