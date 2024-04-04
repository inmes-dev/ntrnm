use std::collections::HashMap;
use crate::ticket::ticket::{SigType, Ticket, TicketManager};
use chrono::Utc;

#[derive(Debug, Clone)]
pub struct SsoSession {
    pub tickets: HashMap<SigType, Ticket>,
    pub msg_cookie: [u8; 4], /// random bytes
    /// Sign server address
    ///
    /// e.g. "https://kritor.support/v9.0.20"
    pub sign_server: String,
}

impl SsoSession {
    pub fn new(
        sign_server: String,
    ) -> Self {
        let msg_cookie = rand::random();
        let sign_server = if sign_server.ends_with("/") {
            sign_server
        } else {
            format!("{}/", sign_server)
        };
        Self {
            tickets: HashMap::new(),
            msg_cookie, sign_server,
        }
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