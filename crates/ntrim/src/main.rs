extern crate pretty_env_logger;
#[macro_use] extern crate log;

use bytes::{BufMut, BytesMut};
use ntrim_net::bytes::BytePacketBuilder;

const WELCOME: &str = r#"
  _   _ _____ ____  ___ __  __
 | \ | |_   _|  _ \|_ _|  \/  |
 |  \| | | | | |_) || || |\/| |
 | |\  | | | |  _ < | || |  | |
 |_| \_| |_| |_| \_\___|_|  |_|
 Welcome to ntrim!"#;

fn main() {
    if let Err(_e) = std::env::var("RUST_LOG") {
        std::env::set_var("RUST_LOG", "info");
    }
    pretty_env_logger::init();
    println!("{}", WELCOME);

    let mut buf = BytesMut::new();
    buf.put_packet(|x| {
        x.put_i32(1);
        x.put_i32(1372362033);
        x.put_packet_with_i32_len(|y| {
            y.put_i32(1);
            y.put_i32(1372362033);
            (y.len() + 4) as i32
        });
    });
    // to hex
    let hex = buf.iter().map(|x| format!("{:02x}", x)).collect::<Vec<String>>().join(" ");
    info!("buf: {:?}", hex);
}

