use std::ops::Deref;
use std::sync::Arc;
use std::sync::atomic::Ordering::SeqCst;
use anyhow::Error;
use tokio::sync::mpsc;
use tokio::sync::mpsc::Sender;
use tokio::sync::mpsc::Receiver;
use tokio::sync::oneshot::error::RecvError;
use ntrim_core::bot::{Bot, BotStatus};
use ntrim_core::commands;
use ntrim_core::events::login_event::LoginResponse;
use crate::config::Config;
use crate::qqsecurity::QSecurityViaHTTP;

mod parser;

pub async fn login_by_session(session_path: String, config: &Config) -> Receiver<LoginResponse> {
    let session = parser::load_session(&session_path);
    let bot = Bot::new(
        session, Arc::new(QSecurityViaHTTP::new(&config.qsign.server))
    ).await.map_err(|e| {
        error!("Failed to create bot session instance: {}", e)
    }).unwrap();
    let (mut tx, rx) = mpsc::channel(1);
    tokio::spawn(async move {
        //let resp_recv = Bot::registerNt(&bot).await.unwrap();
        let resp_recv = Bot::register(&bot).await.unwrap();
        match resp_recv.await {
            Ok(resp) => {
                if let Some(resp) = resp {
                    if resp.msg == "register success" {
                        let mut status = BotStatus::from_bits(bot.status.load(SeqCst)).unwrap();
                        status.set(BotStatus::Online, true);
                        status.set(BotStatus::Offline, false);
                        bot.status.store(status.bits(), SeqCst);
                        info!("Bot register req to online success, Welcome!");
                        tx.send(LoginResponse::Success(bot)).await.map_err(|e| {
                            error!("Failed to send login response: {:?}", e)
                        }).unwrap();
                    }
                } else {
                    error!("Bot register req to online failed, Please check your network connection.");
                    tx.send(LoginResponse::Fail(Error::msg("Bot register req to online failed, Please check your network connection."))).await.unwrap();
                }
            }
            Err(e) => {
                error!("Failed to receive response for register: {:?}", e);
                tx.send(LoginResponse::Fail(Error::new(e))).await.unwrap();
            }
        }
    });
    return rx;
}