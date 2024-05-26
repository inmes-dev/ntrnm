// copy from https://github.com/lz1998/ricq/blob/46c44a3/ricq-core/src/crypto/encrypt.rs
use bytes::{Bytes};
use tokio::sync::OnceCell;
// use openssl::bn::{BigNum, BigNumContext};
// use openssl::ec::{EcGroup, EcPoint, EcKey, PointConversionForm};
// use openssl::nid::Nid;
//use p256::{ecdh::EphemeralSecret, EncodedPoint, PublicKey};

/// 实现Ecdh？这辈子我都懒得实现！
pub static ECDH_VERSION: i32 = 2;

pub async fn ecdh_public_key() -> &'static Vec<u8> {
    static ECDH_PUB_KEY: OnceCell<Vec<u8>> = OnceCell::const_new();
    ECDH_PUB_KEY.get_or_init(|| async {
        hex::decode("04803E9940F3FD8E6474C3CC6994A2F972B1DA6BFDE8DDB4E775E36AB4E439DB8EA7A0E6CAFC722089F4921DFEAEFBA0F56932F3E6AA3ECF81154FD230AF32B18F").unwrap()
    }).await
}

pub async fn ecdh_share_key() -> &'static Vec<u8> {
    static ECDH_SHARE_KEY: OnceCell<Vec<u8>> = OnceCell::const_new();
    ECDH_SHARE_KEY.get_or_init(|| async {
        hex::decode("3F539B2549AB1F71421F2C3A66298D05").unwrap()
    }).await
}

