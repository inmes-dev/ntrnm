use std::ops::Deref;
use std::pin::Pin;
use std::sync::Arc;
use std::sync::atomic::Ordering::SeqCst;
use anyhow::Error;
use tokio::sync::mpsc;
use tokio::sync::mpsc::Sender;
use tokio::sync::mpsc::Receiver;
use tokio::sync::oneshot::error::RecvError;
use ntrim_core::bot::{Bot, BotStatus};
use ntrim_core::{await_response, commands, service};
use ntrim_core::commands::wtlogin::refresh_sig::RefreshSig;
use ntrim_core::commands::wtlogin::wtlogin_request::{WtloginFactory, WtloginBuilder};
use ntrim_core::events::wtlogin_event::WtloginResponse;
use crate::config::Config;
use crate::login::session::register::save_session;
use crate::qqsecurity::QSecurityViaHTTP;

mod register;

pub async fn token_login(session_path: String, config: &Config) -> (Arc<Bot>, Receiver<WtloginResponse>) {
    let session = register::load_session(&session_path);
    let bot = Bot::new(
        session, Arc::new(QSecurityViaHTTP::new(&config.qsign.server))
    ).await.map_err(|e| {
        error!("Failed to create bot session instance: {}", e)
    }).unwrap();
    let result_bot = bot.clone();

    let (mut tx, rx) = mpsc::channel(1);
    tokio::spawn(async move {
        //let resp_recv = Bot::registerNt(&bot).await.unwrap();
       let value =  await_response!(tokio::time::Duration::from_secs(15), async {
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
        });
        match value {
            Ok(resp) => {
                if let Some(resp) = resp {
                    let msg = resp.msg.unwrap_or("protobuf parser error".to_string());
                    if msg == "register success" {
                        info!("Bot register req to online success, Welcome!");

                        info!("RichMedia DownloadRKey: {:?}", service::rich_media::get_download_reky(&bot, 10).await);

                        // 注册退出信号监听器 自动保存会话上下文
                        ntrim_tools::sigint::global_sigint_handler().add_listener(Pin::from(Box::new(async move {
                            info!("Received SIGINT, saving session and exiting");
                            let session = bot.client.session.read().await;
                            save_session(&session_path, session.deref());
                        })));
                        if !tx.is_closed() {
                            tx.send(WtloginResponse::Success()).await.map_err(|e| {
                                error!("Failed to send login response: {:?}", e)
                            }).unwrap();
                        }
                    } else {
                        warn!("Bot register req to online failed: {:?}", msg);
                    }
                } else {
                    error!("Bot register req to online failed, Please check your network connection.");
                    if tx.is_closed() { return }
                    tx.send(WtloginResponse::Fail(Error::msg("Bot register req to online failed, Please check your network connection."))).await.unwrap();
                }
            }
            Err(e) => {
                error!("Failed to receive response for register: {:?}", e);
                if tx.is_closed() { return }
                tx.send(WtloginResponse::Fail(e)).await.unwrap();
            }
        }
    });
    return (result_bot, rx);
}