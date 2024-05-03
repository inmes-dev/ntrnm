use std::error::Error;
use std::fmt;
use std::sync::Arc;
use std::sync::atomic::AtomicU32;
use std::sync::atomic::Ordering::SeqCst;
use bitflags::bitflags;
pub use rc_box::ArcBox;
use tokio::sync::mpsc;
use crate::client::qsecurity::QSecurity;
use crate::client::trpc::TrpcClient;
use crate::events::login_event::LoginResponse;
use crate::events::login_event::LoginResponse::Success;
use crate::servlet::sync_push::SyncPushServlet;
use crate::session::SsoSession;

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct BotStatus: u32 {
        /// 在线
        const Online =  0b00000001;
        /// 离线
        const Offline = 0b00000010;
        /// 冻结
        const Freeze =  0b00000100;
    }
}

pub struct Bot {
    /// TCP client.
    pub client: Arc<TrpcClient>,
    /// Bot status.
    pub status: AtomicU32,
}

impl Bot {
    pub async fn new(
        session: SsoSession,
        qsec_mod: Arc<dyn QSecurity>,
    ) -> Result<Arc<Self>, Box<dyn Error>> {
        let client = TrpcClient::new(session, qsec_mod).await?;
        let bot = Arc::new(Self {
            client,
            status: AtomicU32::new(BotStatus::Offline.bits()),
        });

        SyncPushServlet::register(&bot).await;

        Ok(bot)
    }

    pub fn is_online(&self) -> bool {
        self.client.client.is_connected() &&
            BotStatus::from_bits(self.status.load(SeqCst)).unwrap().contains(BotStatus::Online)
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