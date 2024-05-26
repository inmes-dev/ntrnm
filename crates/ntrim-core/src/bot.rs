use std::{fmt, thread};
use std::ops::Deref;
use std::pin::Pin;
use std::process::exit;
use std::sync::Arc;
use std::sync::atomic::AtomicU32;
use std::sync::atomic::Ordering::SeqCst;
use std::time::Duration;
use anyhow::Error;
use bitflags::bitflags;
use log::{debug, error, info, warn};
use tokio::sync::mpsc;
use ntrim_tools::tokiort;
use crate::await_response;
use crate::client::qsecurity::QSecurity;
use crate::client::trpc::TrpcClient;
use crate::events::wtlogin_event::WtloginResponse;
use crate::events::wtlogin_event::WtloginResponse::Success;
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
    ) -> Result<Arc<Self>, Error> {
        let client = TrpcClient::new(session, qsec_mod).await?;

        let bot = Arc::new(Self {
            client,
            status: AtomicU32::new(BotStatus::Offline.bits()),
        });
        SyncPushServlet::initialize(&bot).await;
        if match option_env!("AUTO_RECONNECT") {
            None => true,
            Some(value) => value == "1",
        } {
            Self::auto_reconnect(&bot).await;
        }

        Ok(bot)
    }

    async fn auto_reconnect(self: &Arc<Self>) {
        let bot = Arc::clone(self);
        tokio::spawn(async move {
            let reconnect_interval = match option_env!("RECONNECT_INTERVAL") {
                None => 5,
                Some(value) => value.parse::<u64>().unwrap_or(5),
            };
            let mut attempt = 0;
            info!("Auto reconnect task started, interval: {}s", reconnect_interval);
            loop {
                tokio::time::sleep(Duration::from_secs(reconnect_interval * ((attempt % 10) + 1))).await;
                if bot.client.is_lost().await {
                    info!("Try to reconnect trpc, attempt: {}", attempt);
                    if let Err(e) = bot.client.try_connect().await {
                        attempt += 1;
                        error!("Failed to reconnect, err: {}", e);
                    } else {
                        info!("Reconnected successfully");
                        attempt = 0;
                        Self::reregister(&bot).await;
                    }
                } else {
                    //debug!("Trpc is connected, skip reconnecting");
                }
            }
        });
    }

    async fn reregister(bot: &Arc<Bot>) {
        match await_response!(Duration::from_secs(15), async {
            let rx = Bot::register(&bot).await;
            if let Some(rx) = rx {
                rx.await.map_err(|e| Error::new(e))
            } else {
                Err(Error::msg("Tcp connection exception"))
            }
        }, |value| {
            Ok(value)
        }, |e| {
            Err(e)
        }) {
            Ok(resp) => {
                if let Some(resp) = resp {
                    let msg = resp.msg.unwrap_or("protobuf parser error".to_string());
                    if msg == "register success" {
                        info!("Bot reregister req to online success, Welcome!");
                    } else {
                        error!("Bot reregister req to online failed: {:?}", msg);
                        exit(0);
                    }
                } else {
                    error!("Bot reregister req to online failed, Please check your network connection.");
                    exit(0);
                }
            }
            Err(e) => {
                error!("Failed to receive response for reregister: {:?}", e);
                exit(0);
            }
        }
        // 上线失败说明当前的session有问题
    }

    pub fn set_online(&self) {
        let mut status = BotStatus::from_bits(self.status.load(SeqCst)).unwrap();
        status.set(BotStatus::Online, true);
        status.set(BotStatus::Offline, false);
        self.status.store(status.bits(), SeqCst);
    }

    pub async fn set_offline(&self) {
        warn!("Bot status change to offline");
        let mut status = BotStatus::from_bits(self.status.load(SeqCst)).unwrap();
        status.set(BotStatus::Online, false);
        status.set(BotStatus::Offline, true);
        self.status.store(status.bits(), SeqCst);
        self.client.set_lost().await;
    }

    pub async fn is_online(&self) -> bool {
        self.client.is_connected().await &&
            BotStatus::from_bits(self.status.load(SeqCst)).unwrap().contains(BotStatus::Online)
    }
}

impl fmt::Debug for Bot {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let uin = tokiort::global_tokio_runtime().block_on(async move {
            self.client.session.read().await.uin
        });
        f.debug_struct("Bot")
            .field("uin", &uin)
            .finish()
    }
}