mod qqsecurity;
mod config;
mod args;
mod login;
mod backend;

extern crate pretty_env_logger;
#[macro_use] extern crate log;

use std::sync::Arc;
use bytes::{BufMut, BytesMut};
use clap::Parser;
use ntrim_core::bot::{ArcBox, Bot};
use ntrim_core::client::qsecurity::QSecurity;
use ntrim_core::session::protocol::QQ_9_0_20;
use ntrim_core::session::SsoSession;
use crate::args::{Args, LoginMode};
use crate::login::session::login_by_session;
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
    let args = Args::parse();
    if let Err(_e) = std::env::var("RUST_LOG") {
        std::env::set_var("RUST_LOG", args.log_level);
    }
    pretty_env_logger::init();
    info!("{}", WELCOME);

    let config = if let Some(path) = args.config_path {
        config::parse_local_config(std::path::PathBuf::from(path))
            .expect("Configuration file parsing failure")
    } else {
        let current_path = std::env::current_dir().unwrap();
        debug!("Current path: {:?}", current_path);
        config::parse_local_config(current_path.join("config.toml"))
            .expect("Configuration file parsing failure")
    };

    match args.login_mode {
        LoginMode::Password { qq, password } => {
            panic!("Password login is not supported yet")
        }
        LoginMode::Session { session_path } => {
            login_by_session(session_path, &config).await;
        }
    }

    if cfg!(feature = "onebot") {
        info!("Using OneBot backend, see https://github.com/botuniverse/onebot");

    } else if cfg!(feature = "kritor") {
        info!("Using Kritor backend, see https://github.com/KarinJS/kritor");

    } else {
        error!("No backend selected, please enable one of the backend features")
    }
    loop {
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    }
}