use std::fmt::Write;
use std::sync::Arc;
use bytes::{BufMut, BytesMut};
use log::{debug, error, info, warn};
use rc_box::ArcBox;
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::sync::{oneshot, RwLock};
use crate::client::codec::decoder::TrpcDecoder;
use crate::client::codec::encoder::TrpcEncoder;
use crate::client::dispatcher::TrpcDispatcher;
use crate::client::packet::from_service_msg::FromServiceMsg;
use crate::client::packet::packet::CommandType::{*};
pub use crate::client::tcp::ClientError;
use crate::client::tcp::{TcpStatus, TcpClient};
use crate::client::packet::packet::UniPacket;
use crate::client::packet::to_service_msg::ToServiceMsg;
use crate::client::qsecurity::QSecurity;
use crate::session::SsoSession;
use crate::session::ticket::{SigType, TicketManager};

pub struct TrpcClient {
    pub(crate) client: TcpClient,
    pub(crate) session: Arc<RwLock<SsoSession>>,
    pub(crate) qsec: Arc<dyn QSecurity>,
    pub(crate) sender: Arc<Sender<ToServiceMsg>>,
    pub(crate) dispatcher: Arc<TrpcDispatcher>
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
        let mut client = if !is_ipv6 {
            TcpClient::new_ipv4_client()
        } else {
            TcpClient::new_ipv6_client()
        };
        match client.connect().await {
            Ok(_) => {
                info!("Connected to server for ({}, {})", session.uin, session.uid)
            }
            Err(e) => {
                info!("Failed to connect to server for ({}, {}): {}", session.uin, session.uid, e);
                return Err(e);
            }
        }
        {
            let mut buf = BytesMut::new();
            buf.put_u32(21);
            buf.put_u32(0x01335239);
            buf.put_u32(0x00000000);
            buf.put_u8(0x4);
            buf.write_str("MSF").unwrap();
            buf.put_u8(0x5);
            buf.put_u32(0x00000000);
            client.write_data(buf).await.unwrap();
        }

        let (tx, rx) = tokio::sync::mpsc::channel(match option_env!("NT_SEND_QUEUE_SIZE") {
            None => 32,
            Some(value) => value.parse::<usize>().unwrap_or(32),
        });
        let trpc = Arc::new(Self {
            client,
            qsec: qsec_mod,
            session: Arc::new(RwLock::new(session)),
            sender: Arc::new(tx),
            dispatcher: Arc::new(TrpcDispatcher::new()),
        });
        TrpcEncoder::init(&trpc, rx);
        TrpcDecoder::init(&trpc);
        trpc.repeat_ping_sign_server();

        Ok(trpc)
    }

    pub fn repeat_ping_sign_server(self: &Arc<Self>) {
        let trpc = Arc::clone(self);
        tokio::spawn(async move {
            let qsec = Arc::clone(&trpc.qsec);
            let mut count = 0;
            while trpc.client.is_connected() {
                let status = qsec.ping().await;
                count += 1;
                if count == 50 {
                    info!("Pinging sign server, status: {}", status);
                    count = 0;
                }
                tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
            }
        });
    }

    pub async fn send_uni_packet(self: &Arc<TrpcClient>, uni_packet: UniPacket) -> Option<oneshot::Receiver<FromServiceMsg>> {
        let (tx, rx) = oneshot::channel();
        let session = self.session.clone();
        let session = session.read().await;
        debug!("Fetch session rwlock: {:?}", session);

        let seq = session.next_seq();
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

        info!("Send packet, seq: {}, cmd: {}", seq, cmd);

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
            WtLoginSt => {
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

    pub async fn disconnect(&mut self) {
        self.client.disconnect().await;
        self.dispatcher.clear().await;
        self.sender.closed().await;
    }
}