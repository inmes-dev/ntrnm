use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::AtomicU32;
use crate::session::ticket::{SigType, Ticket, TicketManager};
use chrono::{DateTime, Local};
use log::{debug, info, warn};
use crate::client::codec::encoder::default_tea_key;
use crate::client::packet::packet::CommandType;
use crate::client::packet::packet::CommandType::{ExchangeSig};
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

    pub encrypt_a1: Vec<u8>,
    pub no_pic_sig: Vec<u8>,
    pub tgtgt_key: Vec<u8>,

    /// refresh sig
    pub wt_session_ticket: Vec<u8>,
    pub wt_session_key: Vec<u8>,
    pub wt_session_create_time: u64,

    pub last_grp_msg_time: u64,
    pub last_c2c_msg_time: u64,

    /// Web Tickets
    pub skey: String,
    pub pskey: String
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
            encrypt_a1: Vec::new(),
            no_pic_sig: Vec::new(),
            tgtgt_key: Vec::new(),
            wt_session_ticket: Vec::new(),
            wt_session_key: Vec::new(),
            wt_session_create_time: 0,
            skey: String::new(),
            pskey: String::new()
        }
    }

    pub fn is_login(&self) -> bool {
        self.contain(SigType::D2)
    }

    pub fn is_online(&self) -> bool {
        self.is_online && self.is_login()
    }

    pub fn get_session_key(&self, command_type: CommandType) -> &[u8] {
        if command_type == ExchangeSig {
            return default_tea_key();
        }
        if let Some(d2) = self.ticket(SigType::D2) {
            d2.sig_key.as_slice()
        } else {
            default_tea_key()
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
        let now = Local::now().timestamp();
        if ticket.id == SigType::D2 || ticket.id == SigType::A2 {
            //let reset_time = now - ticket.expire_time;
            //info!("{:?} ticket inserted, expire after {:.2} days", ticket.id, (reset_time / (60 * 60 * 24)) as f64);
            let expire_time = DateTime::from_timestamp(ticket.expire_time, 0);
            info!("{:?} ticket inserted, expire at {:?}", ticket.id, expire_time);
        }
        if now >= ticket.expire_time && ticket.expire_time != 0 {
            let expire_time = DateTime::from_timestamp(ticket.expire_time, 0);
            warn!("Ticket expired: {:?}, expire_time: {:?}", ticket.id, expire_time);
        }
        debug!("Insert ticket: {:?}", ticket);
        self.tickets.insert(ticket.id, ticket);
    }

    fn ticket(&self, id: SigType) -> Option<&Ticket> {
        self.tickets.get(&id)
    }

    fn ticket_mut(&mut self, id: SigType) -> Option<&mut Ticket> {
        self.tickets.get_mut(&id)
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
            let now = Local::now().timestamp();
            if now >= ticket.expire_time {
                return true;
            }
        }
        true
    }
}