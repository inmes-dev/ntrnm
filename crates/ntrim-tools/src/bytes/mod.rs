mod builder;
mod reader;

use std::string::FromUtf8Error;
use bitflags::bitflags;
use bytes::{Buf, BufMut, BytesMut};
pub use builder::BytePacketBuilder;
pub use reader::BytePacketReader;

bitflags! {
    #[derive(Debug, PartialEq, Eq)]
    pub struct PacketFlag: u32 {
        const I16Len =  0b00000001;
        const I32Len =  0b00000010;
        const I64Len =  0b00000100;
        const ExtraLen = 0b00001000;
    }
}


