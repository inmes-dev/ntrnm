use std::error::Error;
use std::io;
use thiserror::Error;

pub(crate) mod decoder;

pub(crate) mod encoder;
pub(crate) mod qqsecurity;

#[derive(Error, Debug)]
pub(crate) enum CodecError {
    #[error("Packet codec error: {0}")]
    CodecError(Box<dyn Error>),
}

impl From<io::Error> for CodecError {
    fn from(value: io::Error) -> Self {
        CodecError::CodecError(Box::new(value))
    }
}