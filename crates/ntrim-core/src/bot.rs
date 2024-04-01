use std::error::Error;
use ntrim_net::client::Client;

pub struct Bot {
    /// TCP client.
    client: Client,
}

impl Bot {
    pub fn new() -> Result<Self, Box<dyn Error>> {
        let bot = Self {
            client: Client::new_ipv4_client(),
        };

        Ok(bot)
    }


}