mod qqsecurity;

extern crate pretty_env_logger;
#[macro_use] extern crate log;

use std::sync::Arc;
use bytes::{BufMut, BytesMut};
use ntrim_core::bot::{ArcBox, Bot};
use ntrim_core::client::packet::packet::{CommandType, UniPacket};
use ntrim_core::client::qsecurity::QSecurity;
use ntrim_core::sesson::protocol::QQ_9_0_20;
use ntrim_core::sesson::SsoSession;
use crate::qqsecurity::QSecurityViaHTTP;

const WELCOME: &str = r#"
  _   _ _____ ____  ___ __  __
 | \ | |_   _|  _ \|_ _|  \/  |
 |  \| | | | | |_) || || |\/| |
 | |\  | | | |  _ < | || |  | |
 |_| \_| |_| |_| \_\___|_|  |_|
 Welcome to ntrim!"#;

#[tokio::main]
async fn main() {
    if let Err(_e) = std::env::var("RUST_LOG") {
        std::env::set_var("RUST_LOG", "info");
    }
    pretty_env_logger::init();
    println!("{}", WELCOME);

    let buf = vec![1u8];
    let packet = UniPacket::new(
        CommandType::Register,
        "register".to_string(),
        buf.clone(),
    );

    let buf = packet.to_wup_buffer();
    info!("hex: {}", hex::encode(buf.as_ref()));

    let protocol = QQ_9_0_20.clone();
    let bot = Bot::new(
        SsoSession::new(
            (123456789, "test".to_string()),
            protocol,
            Default::default(),
        ),
        Arc::new(QSecurityViaHTTP::new(
            "https://kritor.support/v9.0.20".to_string()
        )),
    ).await.unwrap();
}

