use std::fmt::Display;
use bitflags::bitflags;

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct SigType: u32 {
        const A5 =       0b0000000000000000000000000010; // user_A5
        /// Reserved.
        /// Reserved.
        const A8 =       0b0000000000000000000000010000;
        const STWEB =    0b0000000000000000000000100000;
        const A2 =       0b0000000000000000000001000000; // TGT
        const ST =       0b0000000000000000000010000000;
        /// Reserved.
        const LSKEY =    0b0000000000000000001000000000;
        /// Reserved.
        /// Reserved.
        const SKEY =     0b0000000000000001000000000000;
        const SIG64 =    0b0000000000000010000000000000;
        const OPENKEY =  0b0000000000000100000000000000;
        const TOKEN =    0b0000000000001000000000000000;
        /// Reserved.
        const VKEY =     0b0000000000100000000000000000;
        const D2 =       0b0000000001000000000000000000;
        const SID =      0b0000000010000000000000000000;
        const SuperKey = 0b0000000100000000000000000000;
        const AQSIG =    0b0000001000000000000000000000;
        const LHSIG =    0b0000010000000000000000000000; // useless
        const PAYTOKEN = 0b0000100000000000000000000000;
        const PF =       0b0001000000000000000000000000;
        const DA2 =      0b0010000000000000000000000000;
        const QRPUSH =   0b0100000000000000000000000000;
        const PT4Token = 0b1000000000000000000000000000;
    }
}

#[derive(Debug, Clone)]
pub struct Ticket {
    pub id: SigType,
    /// e.g. skey or d2key
    pub key: Vec<u8>,
    /// e.g. d2
    pub value: Option<Vec<u8>>,
    pub create_time: u64,
    /// 0 means never expire
    /// unit is seconds -> expires after n seconds
    /// `1` means expire after 1 second
    pub expire_time: u32,
}

impl Display for Ticket {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let hex_key = hex::encode(&self.key);
        let hex_value = match &self.value {
            Some(value) => hex::encode(value),
            None => "None".to_string(),
        };
        write!(f, "Ticket {{ id: {:?}, key: {}, value: {}, create_time: {}, expire_time: {} }}", self.id, hex_key, hex_value, self.create_time, self.expire_time)
    }
}

#[macro_export]
macro_rules! create_ticket {
    ($id:expr, $key:expr) => {
        Ticket {
            id: $id,
            key: $key,
            value: None,
        }
    };

    ($id:expr, $key:expr, $value:expr) => {
        Ticket {
            id: $id,
            key: $key,
            value: Some($value),
        }
    };
}

pub trait TicketManager {
    fn insert(&mut self, ticket: Ticket);

    fn ticket(&self, id: SigType) -> Option<&Ticket>;

    fn remove(&mut self, id: SigType) -> Option<Ticket>;

    fn contain(&self, id: SigType) -> bool;

    fn is_expired(&self, id: SigType) -> bool;
}