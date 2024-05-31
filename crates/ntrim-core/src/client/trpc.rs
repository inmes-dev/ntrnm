use std::fmt::Write;
use std::sync::{Arc};
use bytes::{BufMut, BytesMut};
use log::{info, warn};
use tokio::sync::mpsc::{Sender};
use tokio::sync::{RwLock};
use tokio::time::Duration;
use crate::client::codec::decoder::TrpcDecoder;
use crate::client::codec::encoder::TrpcEncoder;
use crate::client::dispatcher::TrpcDispatcher;
use crate::client::tcp::{TcpClient};
use crate::client::packet::to_service_msg::ToServiceMsg;
use crate::client::qsecurity::QSecurity;
use crate::session::SsoSession;
pub use crate::client::tcp::ClientError;

pub struct TrpcClient {
    pub(crate) client: Arc<RwLock<TcpClient>>,
    pub session: Arc<RwLock<SsoSession>>,
    pub qsec: Arc<dyn QSecurity>,
    pub(crate) sender: Arc<Sender<ToServiceMsg>>,
    pub(crate) dispatcher: Arc<TrpcDispatcher>
}

impl TrpcClient {
    pub async fn new(
        session: SsoSession,
        qsec_mod: Arc<dyn QSecurity>
    ) -> Result<Arc<Self>, ClientError> {
        let is_ipv6 = option_env!("IS_NT_IPV6").map_or(false, |v| v == "1");
        let (tx, rx) = tokio::sync::mpsc::channel(
            option_env!("NT_SEND_QUEUE_SIZE")
                .map_or(32, |value| value.parse::<usize>().unwrap_or(32))
        );
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

    pub async fn set_lost(self: &Arc<Self>) {
        let mut client = self.client.write().await;
        client.set_lost().await;
        warn!("Trpc connection lost: network changed or session changed!");
        self.dispatcher.clear_oneshot().await;
    }

    pub async fn disconnect(self: &Arc<Self>) {
        let mut client = self.client.write().await;
        client.disconnect().await;
        self.dispatcher.clear().await;
        self.sender.closed().await;
    }
}
