use std::borrow::Cow;
use crate::codec::qqsecurity::QSecurityResult;
use crate::packet::packet::UniPacket;

#[derive(Debug)]
pub struct ToServiceMsg {
    pub uni_packet: UniPacket,

    pub seq: u32,
    pub first_token: Option<Box<[u8]>>,
    pub second_token: Option<Box<[u8]>>,
    pub(crate) sec_info: Option<QSecurityResult>,
}