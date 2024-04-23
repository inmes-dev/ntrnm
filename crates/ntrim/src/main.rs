mod qqsecurity;
mod config;
mod args;

extern crate pretty_env_logger;
#[macro_use] extern crate log;

use std::sync::Arc;
use bytes::{BufMut, BytesMut};
use clap::Parser;
use futures::io::copy_buf;
use ntrim_core::bot::{ArcBox, Bot};
use ntrim_core::client::qsecurity::QSecurity;
use ntrim_core::sesson::protocol::QQ_9_0_20;
use ntrim_core::sesson::SsoSession;
use crate::args::Args;
use crate::config::parse_local_config_file;
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
        parse_local_config_file(std::path::PathBuf::from(path))
            .expect("Configuration file parsing failure")
    } else {
        let current_path = std::env::current_dir().unwrap();
        debug!("Current path: {:?}", current_path);
        parse_local_config_file(current_path.join("config.toml"))
            .expect("Configuration file parsing failure")
    };


}