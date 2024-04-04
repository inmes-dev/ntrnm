use crate::packet::packet::UniPacket;

#[derive(Debug)]
pub struct FromServiceMsg {
    pub uni_packet: UniPacket,
}