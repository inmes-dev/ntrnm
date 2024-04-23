use std::error::Error;
use std::sync::Arc;
pub use rc_box::ArcBox;
use crate::client::qsecurity::QSecurity;
use crate::client::trpc::TrpcClient;
use crate::sesson::SsoSession;

pub struct Bot {
    /// TCP client.
    client: Arc<TrpcClient>,
}

impl Bot {
    pub async fn new(
        session: SsoSession,
        qsec_mod: Arc<dyn QSecurity>,
    ) -> Result<Self, Box<dyn Error>> {
        let client = TrpcClient::new(session, qsec_mod).await?;
        Ok(Self {
            client
        })
    }


}