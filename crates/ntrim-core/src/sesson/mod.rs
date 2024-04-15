use std::cmp::PartialEq;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::AtomicU32;
use crate::sesson::ticket::{SigType, Ticket, TicketManager};
use chrono::Utc;
use crate::client::codec::encoder::DEFAULT_TEA_KEY;
use crate::client::packet::packet::CommandType;
use crate::client::packet::packet::CommandType::WtLoginSt;
use crate::sesson::device::Device;
use crate::sesson::protocol::Protocol;

pub mod ticket;
pub mod protocol;
pub mod device;

#[derive(Debug, Clone)]
pub struct SsoSession {
    pub uin: u64,
    pub uid: String,

    pub(crate) tickets: HashMap<SigType, Ticket>,
    pub(crate) protocol: Protocol,
    pub(crate) device: Device,

    pub(crate) msg_cookie: [u8; 4], /// random bytes
    pub(crate) ksid: [u8; 16],
    pub(crate) is_online: bool,

    /// sso seq, thread safe
    /// random from 10000 ~ 80000
    pub(crate) sso_seq: Arc<AtomicU32>,
}

impl SsoSession {
    pub fn new(
        account: (u64, String),
        protocol: Protocol,
        device: Device,
    ) -> Self {
        let msg_cookie = rand::random();
        Self {
            uin: account.0,
            uid: account.1,
            tickets: HashMap::new(),
            msg_cookie, protocol, device,
            is_online: false,
            ksid: [0u8; 16],
            sso_seq: Arc::new(AtomicU32::new(
                rand::random::<u32>() % 70000 + 10000
            )),
        }
    }

    pub fn is_login(&self) -> bool {
        self.contain(SigType::D2)
    }

    pub fn is_online(&self) -> bool {
        self.is_online && self.is_login()
    }

    pub fn get_session_key(&self, command_type: CommandType) -> &[u8] {
        if command_type == WtLoginSt {
            return &*DEFAULT_TEA_KEY;
        }
        if let Some(d2) = self.get(SigType::D2) {
            d2.key.as_slice()
        } else {
            &*DEFAULT_TEA_KEY
        }
    }

    pub fn set_ksid(&mut self, ksid: [u8; 16]) {
        self.ksid = ksid;
    }

    pub fn next_seq(&self) -> u32 {
        if self.sso_seq.load(std::sync::atomic::Ordering::SeqCst) > 800_0000 {
            self.sso_seq.store(
                rand::random::<u32>() % 70000 + 10000,
                std::sync::atomic::Ordering::SeqCst
            );
        }
        self.sso_seq.fetch_add(1, std::sync::atomic::Ordering::SeqCst)
    }
}

impl TicketManager for SsoSession {
    fn insert(&mut self, ticket: Ticket) {
        self.tickets.insert(ticket.id, ticket);
    }

    fn get(&self, id: SigType) -> Option<&Ticket> {
        self.tickets.get(&id)
    }

    fn remove(&mut self, id: SigType) -> Option<Ticket> {
        self.tickets.remove(&id)
    }

    fn contain(&self, id: SigType) -> bool {
        if self.tickets.contains_key(&id) {
            true
        } else {
            false
        }
    }

    fn is_expired(&self, id: SigType) -> bool {
        if let Some(ticket) = self.get(id) {
            if ticket.expire_time == 0 {
                return false;
            }
            let now = Utc::now().timestamp() as u64;
            if now - ticket.create_time > ticket.expire_time as u64 {
                return true;
            }
        }
        true
    }
}