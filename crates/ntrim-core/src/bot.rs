use std::error::Error;
use std::fmt;
use std::sync::Arc;
pub use rc_box::ArcBox;
use tokio::sync::mpsc;
use crate::client::qsecurity::QSecurity;
use crate::client::trpc::TrpcClient;
use crate::events::login_event::LoginResponse;
use crate::events::login_event::LoginResponse::Success;
use crate::session::SsoSession;

pub struct Bot {
    /// TCP client.
    pub client: Arc<TrpcClient>,
}

impl Bot {
    pub async fn new(
        session: SsoSession,
        qsec_mod: Arc<dyn QSecurity>,
    ) -> Result<Arc<Self>, Box<dyn Error>> {
        let client = TrpcClient::new(session, qsec_mod).await?;
        Ok(Arc::new(Self {
            client
        }))
    }

    pub fn is_online(&self) -> bool {
        self.client.client.is_connected()
    }
}

impl fmt::Debug for Bot {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let uin = self.client.session.blocking_read().uin;
        f.debug_struct("Bot")
            .field("uin", &uin)
            .finish()
    }
}