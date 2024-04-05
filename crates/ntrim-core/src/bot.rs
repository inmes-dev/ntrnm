use std::error::Error;
use crate::client::Client;
use crate::sesson::protocol::Protocol;
use crate::sesson::SsoSession;

pub struct Bot {
    /// TCP client.
    client: Client,
}

impl Bot {
    pub fn new(
        sso_session: SsoSession
    ) -> Result<Self, Box<dyn Error>> {
        let bot = Self {
            client: Client::new_ipv4_client(sso_session),
        };

        Ok(bot)
    }


}