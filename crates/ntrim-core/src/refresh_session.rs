use std::sync::{Arc, OnceLock};
use log::{error, info, warn};
use crate::bot::Bot;
use crate::commands::wtlogin::wtlogin_request::{WtloginBuilder, WtloginFactory};
use crate::events::wtlogin_event::WtloginResponse;
use crate::session::ticket::{SigType, TicketManager};

impl Bot {
    pub(crate) async fn auto_refresh_session(self: &Arc<Self>) {
        let bot = Arc::clone(self);
        let session = bot.client.session.read().await;
        let d2 = session.ticket(SigType::D2).unwrap();
        if d2.expire_time <= 0 {
            return;
        }
        let refresh_advance_time = option_env!("REFRESH_ADVANCE_TIME")
            .map_or(60 * 60 * 24 * 25, |value|
                value.parse::<i32>().unwrap_or(60 * 60 * 24 * 25)
            );
        let mut interval = (d2.expire_time as i32) - refresh_advance_time;
        let mut fail_time = 0;
        drop(session); // forbid magic error
        tokio::spawn(async move {
            loop {
                if interval > 0 {
                    tokio::time::sleep(tokio::time::Duration::from_secs(interval as u64)).await;
                }
                if !refresh_sig(&bot).await {
                    fail_time += 1;
                    if fail_time == 2 {
                        warn!("Auto refresh sig failed! No more automatic session refreshes!");
                        break;
                    }
                } else {
                    bot.client.set_lost().await;
                    while !Bot::reconnect(&bot).await {
                        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                        warn!("Refresh session successfully, reconnect failed!");
                    }
                }
            }
        });
    }
}

pub async fn refresh_sig(bot: &Arc<Bot>) -> bool {
    let domains = refresh_pskey_domains().clone();
    let rx = WtloginBuilder::build(bot.client.clone(), (16, domains))
        .send().await;
    match rx.await.unwrap() {
        WtloginResponse::Fail(e) => {
            error!("Refresh sig failed: {:?}", e);
            false
        }
        WtloginResponse::Success() | WtloginResponse::RefreshSigSuccess => {
            info!("Refresh sig success");
            true
        }
    }
}

fn refresh_pskey_domains() -> &'static Vec<String> {
    static DOMAINS: OnceLock<Vec<String>> = OnceLock::new();
    DOMAINS.get_or_init(|| {
        vec![
            "office.qq.com".to_string(),
            "qun.qq.com".to_string(),
            "gamecenter.qq.com".to_string(),
            "docs.qq.com".to_string(),
            "mail.qq.com".to_string(),
            "tim.qq.com".to_string(),
            "ti.qq.com".to_string(),
            "vip.qq.com".to_string(),
            "tenpay.com".to_string(),
            "qqweb.qq.com".to_string(),
            "qzone.qq.com".to_string(),
            "mma.qq.com".to_string(),
            "game.qq.com".to_string(),
            "openmobile.qq.com".to_string(),
            "connect.qq.com".to_string()
        ]
    })
}