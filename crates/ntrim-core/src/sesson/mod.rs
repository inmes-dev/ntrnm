use std::collections::HashMap;
use crate::sesson::ticket::{SigType, Ticket, TicketManager};
use chrono::Utc;
use crate::codec::encoder::DEFAULT_TEA_KEY;
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

    /// Sign server address
    ///
    /// e.g. "https://kritor.support/v9.0.20"
    pub(crate) sign_server: String,
}

impl SsoSession {
    pub fn new(
        account: (u64, String),
        protocol: Protocol,
        device: Device,
        sign_server: String,
    ) -> Self {
        let msg_cookie = rand::random();
        let sign_server = if sign_server.ends_with("/") {
            sign_server
        } else {
            format!("{}/", sign_server)
        };
        Self {
            uin: account.0,
            uid: account.1,
            tickets: HashMap::new(),
            msg_cookie, protocol, device,
            is_online: false,
            sign_server,
            ksid: [0u8; 16],
        }
    }

    pub fn is_login(&self) -> bool {
        self.contain(SigType::D2)
    }

    pub fn is_online(&self) -> bool {
        self.is_online && self.is_login()
    }

    pub fn get_session_key(&self) -> &[u8] {
        if let Some(d2) = self.get(SigType::D2) {
            d2.key.as_slice()
        } else {
            &*DEFAULT_TEA_KEY
        }
    }

    pub fn set_ksid(&mut self, ksid: [u8; 16]) {
        self.ksid = ksid;
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