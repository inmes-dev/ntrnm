use crate::client::packet::packet::UniPacket;
use crate::client::qsecurity::QSecurityResult;

#[derive(Debug)]
pub(crate) struct ToServiceMsg {
    pub uni_packet: UniPacket,
    pub seq: u32,
    pub first_token: Option<Box<[u8]>>,
    pub second_token: Option<Box<[u8]>>,
    pub(crate) sec_info: Option<QSecurityResult>,
}

impl ToServiceMsg {
    pub fn new(uni_packet: UniPacket, seq: u32) -> Self {
        Self {
            uni_packet,
            seq,
            first_token: None,
            second_token: None,
            sec_info: None,
        }
    }
}