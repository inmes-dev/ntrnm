use std::error::Error;
use std::fmt::{Display, Formatter};

#[derive(Debug)]
pub enum ClientError {
    DNSQueryError,
    TcpConnectError,
    TcpNotConnectedError,
}

impl Display for ClientError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ClientError::DNSQueryError => write!(f, "DNS query error"),
            ClientError::TcpConnectError => write!(f, "TCP connect error"),
            ClientError::TcpSetOptionsError => write!(f, "Cannot set options for TCP stream"),
            ClientError::TcpNotConnectedError => write!(f, "TCP stream is not connected"),
        }
    }
}

impl Error for ClientError {}
