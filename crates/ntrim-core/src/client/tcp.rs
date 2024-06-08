use std::net::SocketAddr;
use std::sync::Arc;
use std::sync::atomic::AtomicU32;
use std::sync::atomic::Ordering::SeqCst;
use anyhow::Error;
use bitflags::bitflags;
use bytes::BytesMut;
use log::{debug, error, info};
use thiserror::Error;
use tokio::io::{AsyncWriteExt, Interest};
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
use tokio::net::{TcpSocket, TcpStream};
use tokio::sync::mpsc::Sender;
use tokio::sync::Mutex;
use url::Host::Ipv4;
use crate::client::packet::packet::UniPacket;
use crate::session::SsoSession;

const NT_V4_SERVER: &str = "msfwifi.3g.qq.com";
const NT_V6_SERVER: &str = "msfwifiv6.3g.qq.com";

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct TcpStatus: u32 {
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
        /// Loss of connection.
        const Lost =            0b01000000;
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
    WriteError(Error),
    #[error("TCP read error: {0}")]
    ReadError(Error),
}

type TrpcWriteChannel = Arc<Mutex<OwnedWriteHalf>>;
type TrpcReadChannel = Arc<Mutex<OwnedReadHalf>>;

#[derive(Debug)]
pub(crate) struct TcpClient {
    status: AtomicU32,
    pub(crate) channel: (Option<TrpcWriteChannel>, Option<TrpcReadChannel>),
    addr: Option<SocketAddr>
}

impl TcpClient {
    pub fn new_ipv6_client() -> Self {
        Self {
            status: AtomicU32::new((TcpStatus::Ipv6Addr | TcpStatus::Ready).bits()),
            channel: (None, None),
            addr: None
        }
    }

    pub fn new_ipv4_client() -> Self {
        Self {
            status: AtomicU32::new((TcpStatus::Ipv4Addr | TcpStatus::Ready).bits()),
            channel: (None, None),
            addr: None
        }
    }

    async fn query_for_address(&self) -> Result<Vec<SocketAddr>, ClientError> {
        let status = TcpStatus::from_bits(self.status.load(SeqCst)).unwrap();
        let server = if status.contains(TcpStatus::Ipv4Addr) {
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
        if self.addr.is_none() {
            let mut addrs = self.query_for_address().await?;
            let addr = addrs.swap_remove(0);
            self.addr = Some(addr);
        }

        let mut status = TcpStatus::from_bits(self.status.load(SeqCst)).unwrap();

        //info!("Connecting to server: {:?}", self.addr);
        let tcp = if status.contains(TcpStatus::Ipv4Addr) {
            TcpSocket::new_v4()
        } else {
            TcpSocket::new_v6()
        }.unwrap();
        let mut tcp_stream = match tcp.connect(self.addr.unwrap()).await {
            Ok(result) => result,
            Err(e) => {
                error!("Failed to connect server: {}", e);
                return Err(ClientError::ConnectError);
            }
        };

        let (rx, tx) = tcp_stream.into_split();
        self.channel = (Some(Arc::new(Mutex::new(tx))), Some(Arc::new(Mutex::new(rx))));

        status.set(TcpStatus::Ready, true);
        status.set(TcpStatus::Connected, true);
        status.set(TcpStatus::Disconnected, false);
        status.set(TcpStatus::Lost, false);
        self.status.store(status.bits(), SeqCst);

        info!("Connected to server: {:?}", self.addr);

        Ok(())
    }

    pub(crate) fn reader(&self) -> TrpcReadChannel {
        Arc::clone(self.channel.1.as_ref().unwrap())
    }

    pub(crate) async fn write_data(&self, mut data: BytesMut) -> Result<(), ClientError> {
        if !self.is_connected() {
            return Err(ClientError::NotConnectError);
        }
        let tx = self.channel.0.as_ref().unwrap().clone();
        let mut guard = tx.lock().await;
        if let Err(e) = guard.writable().await {
            error!("Tcp stream is not writable: {:?}", e);
            return Err(ClientError::NotConnectError);
        }

        //info!("Writing data to server: {}", hex::encode(data.as_ref()));

        if let Err(e) = guard.write_all_buf(&mut data).await {
            error!("Failed to write data to server: {:?}", e);
            return Err(ClientError::WriteError(Error::new(e)));
        } else {
            debug!("Data written to server: {}", data.len());
            Ok(())
        }
    }

    pub fn is_connected(&self) -> bool {
        let status = TcpStatus::from_bits(self.status.load(SeqCst)).unwrap();
        status.contains(TcpStatus::Connected)
    }

    pub fn is_lost(&self) -> bool {
        let status = TcpStatus::from_bits(self.status.load(SeqCst)).unwrap();
        status.contains(TcpStatus::Lost)
    }

    pub(crate) async fn set_lost(&mut self) {
        let mut status = TcpStatus::from_bits(self.status.load(SeqCst)).unwrap();
        status.set(TcpStatus::Connected, false);
        status.set(TcpStatus::Disconnected, false);
        status.set(TcpStatus::Ready, true);
        status.set(TcpStatus::Lost, true);
        self.status.store(status.bits(), SeqCst);

        if let Some(mut tx) = self.channel.0.take() {
            let mut guard = tx.lock().await;
            let result = guard.shutdown().await;
            if let Err(e) = result {
                error!("Failed to shutdown tcp client: {}", e);
            } else {
                info!("Tcp client shutdown successfully, because the connection is lost");
            }
        }
        self.channel = (None, None);
    }

    /// close the connection, please use std::drop instead of this method.
    pub(crate) async fn disconnect(&mut self) {
        let mut status = TcpStatus::from_bits(self.status.load(SeqCst)).unwrap();
        status.set(TcpStatus::Connected, false);
        status.set(TcpStatus::Disconnected, true);
        status.set(TcpStatus::Ready, false);
        status.set(TcpStatus::Lost, false);
        self.status.store(status.bits(), SeqCst);

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
