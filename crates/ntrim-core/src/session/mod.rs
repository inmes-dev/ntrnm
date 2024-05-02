use std::cmp::PartialEq;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::AtomicU32;
use crate::session::ticket::{SigType, Ticket, TicketManager};
use chrono::Utc;
use log::{debug, info, warn};
use crate::client::codec::encoder::DEFAULT_TEA_KEY;
use crate::client::packet::packet::CommandType;
use crate::client::packet::packet::CommandType::WtLoginSt;
use crate::session::device::Device;
use crate::session::protocol::Protocol;

pub mod ticket;
pub mod protocol;
pub mod device;

#[derive(Debug, Clone)]
pub struct SsoSession {
    pub uin: u64,
    pub uid: String,

    pub tickets: HashMap<SigType, Ticket>,
    pub protocol: Protocol,
    pub device: Device,

    pub msg_cookie: [u8; 4], /// random bytes
    pub ksid: [u8; 16],
    pub guid: [u8; 16],
    pub is_online: bool,

    /// sso seq, thread safe
    /// random from 10000 ~ 80000
    pub sso_seq: Arc<AtomicU32>,

    pub last_grp_msg_time: u64,
    pub last_c2c_msg_time: u64,
}

impl SsoSession {
    pub fn new(
        account: (u64, String),
        protocol: Protocol,
        device: Device,
        ksid: [u8; 16],
        guid: [u8; 16],
    ) -> Self {
        let msg_cookie = rand::random();
        Self {
            uin: account.0,
            uid: account.1,
            tickets: HashMap::new(),
            msg_cookie, protocol, device,
            is_online: false,
            ksid, guid,
            sso_seq: Arc::new(AtomicU32::new(
                rand::random::<u32>() % 70000 + 20000
            )),
            last_c2c_msg_time: 0,
            last_grp_msg_time: 0,
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
        if let Some(d2) = self.ticket(SigType::D2) {
            d2.sig_key.as_slice()
        } else {
            &*DEFAULT_TEA_KEY
        }
    }

    pub fn next_seq(&self) -> u32 {
        if self.sso_seq.load(std::sync::atomic::Ordering::SeqCst) > 800_0000 {
            self.sso_seq.store(
                rand::random::<u32>() % 70000 + 20000,
                std::sync::atomic::Ordering::SeqCst
            );
        }
        self.sso_seq.fetch_add(1, std::sync::atomic::Ordering::SeqCst)
    }
}

impl TicketManager for SsoSession {
    fn insert(&mut self, ticket: Ticket) {
        if ticket.id == SigType::D2 || ticket.id == SigType::ST || ticket.id == SigType::A2 {
            info!("{:?} ticket inserted, expire after {:.2} days", ticket.id, ticket.expire_time as f64 / (60 * 60 * 24) as f64);
        }
        let now = Utc::now().timestamp();
        if ticket.expire_time != 0 && now as u64 >= ticket.expire_time as u64 + ticket.create_time {
            warn!("Ticket expired: {:?}", ticket.id);
        }
        debug!("Insert ticket: {:?}", ticket);
        self.tickets.insert(ticket.id, ticket);
    }

    fn ticket(&self, id: SigType) -> Option<&Ticket> {
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
        if let Some(ticket) = self.ticket(id) {
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