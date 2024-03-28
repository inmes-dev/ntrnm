use std::error::Error;
use std::net::SocketAddr;

use serde::{Deserialize, Serialize};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

use crate::config::Config;
use crate::error::ClientError;
use crate::global::NT_SERVER;

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub struct Status {
    connected: bool,
}

#[derive(Debug)]
pub struct Client {
    config: Config,
    status: Status,
    stream: Option<TcpStream>,
}

impl Client {
    pub fn new(config: Config) -> Self {
        Self {
            config,
            status: Status { connected: false },
            stream: None,
        }
    }

    async fn query_for_address(&self) -> Result<Vec<SocketAddr>, Box<dyn Error>> {
        Ok(tokio::net::lookup_host(NT_SERVER).await?.collect())
    }

    pub async fn connect(&mut self) -> Result<(), ClientError> {
        let addresses = self
            .query_for_address()
            .await
            .map_err(|_| ClientError::DNSQueryError)?;

        if addresses.is_empty() {
            return Err(ClientError::DNSQueryError);
        }

        let tcp_stream = TcpStream::connect(&addresses[0])
            .await
            .map_err(|_| ClientError::TcpConnectError)?;

        self.stream = Some(tcp_stream);
        self.status.connected = true;

        Ok(())
    }

    pub fn is_connected(&self) -> bool {
        self.status.connected
    }

    pub async fn disconnect(&mut self) {
        if let Some(stream) = &mut self.stream.take() {
            let _ = stream.shutdown();
        }

        self.stream = None;

        self.status.connected = false;
    }

    pub async fn send(&mut self, data: &[u8]) -> Result<(), Box<dyn Error>> {
        if !self.status.connected {
            return Err(Box::new(ClientError::TcpConnectError));
        }

        if let Some(stream) = &mut self.stream {
            stream.write_all(data).await?;
        }

        Ok(())
    }

    pub async fn receive(&mut self, data: &mut [u8]) -> Result<(), Box<dyn Error>> {
        if !self.status.connected {
            return Err(Box::new(ClientError::TcpConnectError));
        }

        if let Some(stream) = &mut self.stream {
            stream.read_exact(data).await?;
        }

        Ok(())
    }
}
