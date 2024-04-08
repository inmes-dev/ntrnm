mod qsec;

use std::error::Error;
use std::future::Future;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use bitflags::bitflags;
use bytes::Buf;
use log::{error, info};

use serde::{Deserialize, Serialize};
use thiserror::Error;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
use tokio::net::TcpStream;
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
    DnsQueryError,
    #[error("TCP connect error")]
    TcpConnectError,
    #[error("TCP not connected")]
    TcpNotConnectedError,
    #[error("TCP write error: {0}")]
    TcpWriteError(Box<dyn Error>),
    #[error("TCP read error: {0}")]
    TcpReadError(Box<dyn Error>),
}

#[derive(Debug)]
pub(crate) struct Client {
    status: Status,
    channel: (Option<Arc<Mutex<OwnedWriteHalf>>>, Option<OwnedReadHalf>),
    pub session: SsoSession
}

impl Client {
    pub fn new_ipv6_client(
        session: SsoSession
    ) -> Self {
        Self {
            status: Status::Ipv6Addr | Status::Ready,
            channel: (None, None),
            session
        }
    }

    pub fn new_ipv4_client(
        session: SsoSession
    ) -> Self {
        Self {
            status: Status::Ipv4Addr | Status::Ready,
            channel: (None, None),
            session
        }
    }

    async fn query_for_address(&self) -> Result<Vec<SocketAddr>, ClientError> {
        let server = if self.status.contains(Status::Ipv4Addr) {
            NT_V4_SERVER
        } else {
            NT_V6_SERVER
        };
        info!("Querying for address: {}", server);
        let addrs: Vec<SocketAddr> = match tokio::net::lookup_host(server).await {
            Ok(result) => result.collect(),
            Err(e) => {
                error!("Failed to query for address: {}", e);
                return Err(ClientError::DnsQueryError);
            }
        };
        return if addrs.is_empty() {
            Err(ClientError::DnsQueryError)
        } else {
            Ok(addrs)
        }
    }

    pub async fn connect(&mut self) -> Result<(), ClientError> {
        let addrs = self.query_for_address().await?;

        let mut tcp_stream = match TcpStream::connect(addrs.first().unwrap()).await {
            Ok(result) => result,
            Err(e) => {
                error!("Failed to connect server: {}", e);
                return Err(ClientError::TcpConnectError);
            }
        };

        let (rx, tx) = tcp_stream.into_split();

        // `tcp_stream` is moved into `rx` and `tx` so it's useless now.
        self.channel = (Some(Arc::new(Mutex::new(tx))), Some(rx));
        self.status.set(Status::Ready, true);
        self.status.set(Status::Connected, true);
        self.status.set(Status::Disconnected, false);

        Ok(())
    }

    async fn on_receive(&mut self) {
        let mut rx = self.channel.1.as_mut()
            .ok_or(ClientError::TcpNotConnectedError)?;
        let mut buf = vec![0u8; 1024];

    }

    pub fn is_connected(&self) -> bool {
        self.status.contains(Status::Connected)
    }

    /// close the connection, please use std::drop instead of this method.
    fn disconnect(&mut self) {
        self.status.set(Status::Connected, false);
        self.status.set(Status::Disconnected, true);
        self.status.set(Status::Ready, false);
    }
}

impl Drop for Client {
    fn drop(&mut self) {
        self.disconnect();
        if let Some(mut tx) = self.channel.0.take() {
            match tx.lock() {
                Ok(mut guard) => {
                    let _ = guard.shutdown();
                },
                Err(poisoned) => {
                    let mut guard = poisoned.into_inner();
                    let _ = guard.shutdown();
                }
            }
        }
        self.channel = (None, None);
    }
}