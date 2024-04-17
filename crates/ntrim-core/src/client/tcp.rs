use std::error::Error;
use std::net::SocketAddr;
use std::sync::Arc;
use bitflags::bitflags;
use bytes::BytesMut;
use log::{error, info};
use thiserror::Error;
use tokio::io::AsyncWriteExt;
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
use tokio::net::TcpStream;
use tokio::sync::mpsc::Sender;
use tokio::sync::Mutex;
use crate::client::packet::packet::UniPacket;
use crate::sesson::SsoSession;

const NT_V4_SERVER: &str = "msfwifi.3g.qq.com";
const NT_V6_SERVER: &str = "msfwifiv6.3g.qq.com";

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct Status: u32 {
        /// The client has an IPv6 address.
        const Ipv6Addr =       0b00000001;
        /// The client has an IPv4 address.
        const Ipv4Addr =       0b00000010;
        /// The client is connected to the server.
        const Connected =       0b00000100;
        /// The client is disconnected from the server.
        const Disconnected =    0b00001000;
        /// The client is connecting to the server.
        const Connecting =      0b00010000;
        /// The client is ready to connect.
        const Ready =           0b00100000;
    }
}

#[derive(Error, Debug)]
pub enum ClientError {
    #[error("DNS query error")]
    QueryDnsError,
    #[error("TCP connect error")]
    ConnectError,
    #[error("TCP not connected")]
    NotConnectError,
    #[error("TCP write error: {0}")]
    WriteError(Box<dyn Error>),
    #[error("TCP read error: {0}")]
    ReadError(Box<dyn Error>),
}

type TrpcWriteChannel = Arc<Mutex<OwnedWriteHalf>>;
type TrpcReadChannel = Arc<Mutex<OwnedReadHalf>>;

#[derive(Debug)]
pub(crate) struct TcpClient {
    pub(crate) status: Status,
    pub(crate) channel: (Option<TrpcWriteChannel>, Option<TrpcReadChannel>),
}

impl TcpClient {
    pub fn new_ipv6_client() -> Self {
        Self {
            status: Status::Ipv6Addr | Status::Ready,
            channel: (None, None),
        }
    }

    pub fn new_ipv4_client() -> Self {
        Self {
            status: Status::Ipv4Addr | Status::Ready,
            channel: (None, None),
        }
    }

    async fn query_for_address(&self) -> Result<Vec<SocketAddr>, ClientError> {
        let server = if self.status.contains(Status::Ipv4Addr) {
            NT_V4_SERVER
        } else {
            NT_V6_SERVER
        };
        info!("Querying for address: {}", server);
        let addrs: Vec<SocketAddr> = match tokio::net::lookup_host((server, 8080)).await {
            Ok(result) => result.collect(),
            Err(e) => {
                error!("Failed to query for address: {}", e);
                return Err(ClientError::QueryDnsError);
            }
        };
        return if addrs.is_empty() {
            Err(ClientError::QueryDnsError)
        } else {
            Ok(addrs)
        }
    }

    pub(crate) async fn connect(&mut self) -> Result<(), ClientError> {
        let addrs = self.query_for_address().await?;
        let addr = addrs.first().unwrap();
        info!("Connecting to server: {}", addr);
        let mut tcp_stream = match TcpStream::connect(addr).await {
            Ok(result) => result,
            Err(e) => {
                error!("Failed to connect server: {}", e);
                return Err(ClientError::ConnectError);
            }
        };

        let (rx, tx) = tcp_stream.into_split();

        // `tcp_stream` is moved into `rx` and `tx` so it's useless now.
        self.channel = (Some(Arc::new(Mutex::new(tx))), Some(Arc::new(Mutex::new(rx))));
        self.status.set(Status::Ready, true);
        self.status.set(Status::Connected, true);
        self.status.set(Status::Disconnected, false);

        Ok(())
    }

    pub(crate) fn reader(&self) -> TrpcReadChannel {
        Arc::clone(self.channel.1.as_ref().unwrap())
    }

    pub(crate) async fn write_data(&self, mut data: BytesMut) -> Result<(), ClientError> {
        if !self.is_connected() {
            return Err(ClientError::NotConnectError);
        }
        let tx = self.channel.0.as_ref().unwrap();
        let mut guard = tx.lock().await;
        match guard.write_all_buf(&mut data).await {
            Ok(_) => Ok(()),
            Err(e) => {
                error!("Failed to write data to tcp stream: {}", e);
                Err(ClientError::WriteError(Box::new(e)))
            }
        }
    }

    pub fn is_connected(&self) -> bool {
        self.status.contains(Status::Connected)
    }

    /// close the connection, please use std::drop instead of this method.
    pub(crate) async fn disconnect(&mut self) {
        self.status.set(Status::Connected, false);
        self.status.set(Status::Disconnected, true);
        self.status.set(Status::Ready, false);
        if let Some(mut tx) = self.channel.0.take() {
            let mut guard = tx.lock().await;
            let result = guard.shutdown().await;
            if let Err(e) = result {
                error!("Failed to shutdown tcp client: {}", e);
            } else {
                info!("Tcp client shutdown successfully");
            }
        }
        self.channel = (None, None);
    }
}
