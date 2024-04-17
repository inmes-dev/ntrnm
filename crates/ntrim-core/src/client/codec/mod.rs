use std::error::Error;
use std::io;
use thiserror::Error;

pub(crate) mod decoder;

pub(crate) mod encoder;


#[derive(Error, Debug)]
pub enum CodecError {
    #[error("Packet codec error: {0}")]
    CodecError(Box<dyn Error>),
    #[error("Tea_key length is invalid")]
    InvalidTeaKey,
    #[error("IO error")]
    IoError,
    #[error("Not connect to server")]
    NotConnectError
}

impl From<io::Error> for CodecError {
    fn from(value: io::Error) -> Self {
        CodecError::CodecError(Box::new(value))
    }
}