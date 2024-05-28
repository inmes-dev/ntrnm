use std::process::exit;
use std::sync::Arc;
use std::time::Duration;
use anyhow::Error;
use log::{error, info, warn};
use crate::await_response;
use crate::bot::Bot;

impl Bot {
    pub(crate) async fn auto_reconnect(self: &Arc<Self>) {
        let bot = Arc::clone(self);
        tokio::spawn(async move {
            let reconnect_interval = option_env!("RECONNECT_INTERVAL").map_or(5, |value| value.parse::<u64>().unwrap_or(5));
            let mut attempt = 0;
            info!("Auto reconnect task started, interval: {}s", reconnect_interval);
            loop {
                tokio::time::sleep(Duration::from_secs(reconnect_interval * ((attempt % 10) + 1))).await;
                if bot.client.is_lost().await {
                    info!("Try to reconnect trpc, attempt: {}", attempt);
                    if Self::reconnect(&bot).await {
                        attempt = 0;
                    } else {
                        attempt += 1;
                    }
                } else {
                    //debug!("Trpc is connected, skip reconnecting");
                }
            }
        });
    }

    pub(crate) async fn reconnect(bot: &Arc<Bot>) -> bool {
        if let Err(e) = bot.client.try_connect().await {
            error!("Failed to reconnect, err: {}", e);
            false
        } else {
            info!("Reconnected successfully");
            Self::reregister(&bot).await;
            true
        }
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
                    warn!("Bot reregister req to online failed, Please check your network connection.");
                    bot.client.set_lost().await;
                }
            }
            Err(e) => {
                warn!("Failed to receive response for reregister: {:?}", e);
                bot.client.set_lost().await;
            }
        }
        // 上线失败说明当前的session有问题
    }
}