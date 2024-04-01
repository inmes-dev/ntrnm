use std::error::Error;
use std::fmt::{Display, Formatter};

#[derive(Debug)]
pub enum ClientError {
    DNSQueryError,
    TcpConnectError,
    TcpNotConnectedError,
    TcpWriteError(Box<dyn Error>),
    TcpReadError(Box<dyn Error>),
}

impl Display for ClientError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ClientError::DNSQueryError => write!(f, "DNS query error"),
            ClientError::TcpConnectError => write!(f, "TCP connect error"),
            ClientError::TcpNotConnectedError => write!(f, "TCP stream is not connected"),
            ClientError::TcpWriteError(e) => write!(f, "TCP write error: {e}"),
            ClientError::TcpReadError(e) => write!(f, "TCP read error: {e}"),
        }
    }
}

impl Error for ClientError {}
