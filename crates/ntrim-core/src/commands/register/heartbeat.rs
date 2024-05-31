use std::sync::Arc;
use std::time::Duration;
use anyhow::Error;
use chrono::Local;
use log::{error, info};
use tokio::time::{Instant, interval_at};
use crate::{await_response, commands};
use crate::bot::Bot;

pub(crate) static mut LAST_PACKET_TIME: i64 = 0i64;

impl Bot {
    pub(crate) fn do_heartbeat(bot: Arc<Bot>) {
        let heartbeat_interval = option_env!("HEARTBEAT_INTERVAL")
            .unwrap_or("270").parse::<u64>().unwrap();
        tokio::spawn(async move {
            let start = Instant::now() + Duration::from_secs(heartbeat_interval);
            let interval = Duration::from_secs(heartbeat_interval);
            let mut intv = interval_at(start, interval);
            loop {
                intv.tick().await;
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

                unsafe {
                    intv = if LAST_PACKET_TIME + 60 < Local::now().timestamp() {
                        interval_at(start, Duration::from_secs(10))
                    } else {
                        interval_at(start, interval)
                    }
                }
            }
        });
    }
}