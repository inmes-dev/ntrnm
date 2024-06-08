use std::sync::Arc;
use std::time::Duration;
use anyhow::Error;
use chrono::Local;
use log::{error, info};
use tokio::time::{Instant, interval_at};
use crate::{await_response, commands};
use crate::bot::Bot;
use crate::client::codec::LAST_PACKET_TIME;

impl Bot {
    pub(crate) fn do_heartbeat(bot: Arc<Bot>) {
        let heartbeat_interval = option_env!("HEARTBEAT_INTERVAL")
            .unwrap_or("270").parse::<u64>().unwrap();
        tokio::spawn(async move {
            let no_packet_interval = Duration::from_secs(10);
            let nt_interval = Duration::from_secs(heartbeat_interval);
            loop {
                unsafe {
                    if LAST_PACKET_TIME == 0 || (LAST_PACKET_TIME + 60 >= Local::now().timestamp()) {
                        tokio::time::sleep(nt_interval).await;
                    } else {
                        tokio::time::sleep(no_packet_interval).await;
                    }
                }
                if !bot.is_online().await { break; }

                let is_success = await_response!(Duration::from_secs(5),
                    async {
                        let rx = Bot::send_nt_heartbeat(&bot).await;
                        if let Some(rx) = rx {
                            rx.await.map_err(|e| Error::new(e))
                        } else {
                            Err(Error::msg("Tcp connection exception"))
                        }
                    }, |value| {
                        info!("Bot heartbeat sent successfully! Next internal: {:?}", value);
                        true
                    }, |err| {
                        error!("Bot heartbeat sent failed! Error: {:?}", err);
                        false
                    }
                );
                if is_success {
                    Bot::send_heartbeat(&bot).await;
                } else {
                    bot.set_offline().await;
                }
            }
        });
    }
}