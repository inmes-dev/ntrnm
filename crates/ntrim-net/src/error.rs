use std::error::Error;
use std::fmt::{Display, Formatter};
use std::io;
use thiserror::Error;

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

#[derive(Error, Debug)]
pub enum CodecError {
    #[error("Packet codec error: {0}")]
    CodecError(Box<dyn Error>),

}

impl From<io::Error> for CodecError {
    fn from(value: io::Error) -> Self {
        CodecError::CodecError(Box::new(value))
    }
}