use std::sync::{Arc, Mutex};
use log::{debug, info};
use rc_box::ArcBox;
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::sync::RwLock;
use crate::client::codec::decoder::TrpcDecoder;
use crate::client::codec::encoder::TrpcEncoder;
use crate::client::dispatcher::TrpcDispatcher;
pub use crate::client::tcp::ClientError;
use crate::client::tcp::{Status, TcpClient};
use crate::client::packet::packet::UniPacket;
use crate::client::packet::to_service_msg::ToServiceMsg;
use crate::client::qsecurity::QSecurity;
use crate::sesson::SsoSession;

pub struct TrpcClient {
    pub(crate) client: TcpClient,
    pub(crate) session: Arc<RwLock<SsoSession>>,
    pub(crate) qsec: Arc<dyn QSecurity>,
    pub(crate) sender: Arc<Sender<ToServiceMsg>>,
    pub(crate) dispenser: Arc<TrpcDispatcher>
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

        let (tx, rx) = tokio::sync::mpsc::channel(match option_env!("NT_SEND_QUEUE_SIZE") {
            None => 32,
            Some(value) => value.parse::<usize>().unwrap_or(32),
        });
        let trpc = Arc::new(Self {
            client,
            qsec: qsec_mod,
            session: Arc::new(RwLock::new(session)),
            sender: Arc::new(tx),
            dispenser: Arc::new(TrpcDispatcher::new()),
        });
        TrpcEncoder::init(&trpc, rx);
        TrpcDecoder::init(&trpc);

        Ok(trpc)
    }



    pub fn send(uni: UniPacket) {

    }

    pub async fn disconnect(&mut self) {
        self.client.disconnect().await;
        self.sender.closed().await;
    }
}