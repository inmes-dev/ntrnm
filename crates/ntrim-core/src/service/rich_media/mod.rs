use std::collections::HashMap;
use std::sync::Arc;
use anyhow::Error;
use tokio::sync::Mutex;
use once_cell::unsync::Lazy;
use crate::await_response;
use crate::bot::Bot;

#[derive(Debug, Clone)]
pub struct RKey {
    pub flag: u8,
    pub key: String,
    pub ttl: i32,
    pub expire_time: i64,
}

impl RKey {
    pub fn new(flag: u8, key: String, ttl: i32, expire_time: i64) -> Self {
        Self {
            flag,
            key,
            ttl,
            expire_time,
        }
    }

    pub fn is_expired(&self) -> bool {
        self.expire_time < chrono::Local::now().timestamp()
    }
}

static mut RKEY: Lazy<Mutex<HashMap<u8, RKey>>> = Lazy::new(|| Mutex::new(HashMap::new()));

pub async fn get_download_reky(bot: &Arc<Bot>, flag: u8) -> Result<Option<RKey>, Error> {
    let refresh_rkey = || async {
        let value =  await_response!(tokio::time::Duration::from_secs(5), async {
            let rx = Bot::request_download_rkey(bot).await;
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
        value
    };
    let is_rkey_expires = unsafe { RKEY.lock().await.get(&flag).cloned() }
        .map_or(true, |v| v.is_expired());
    if is_rkey_expires {
        let rsp = match refresh_rkey().await? {
            Some(rsp) => rsp,
            None => return Err(Error::msg("DownloadRKeyRsp is none")),
        };;
        unsafe {
            for key in rsp.rkeys {
                let r#type = key.r#type.unwrap() as u8;
                RKEY.lock().await.insert(r#type, RKey::new(
                    r#type, key.rkey, key.rkey_ttl_sec as i32, key.rkey_create_time.unwrap() as i64 + key.rkey_ttl_sec as i64
                ));
            }
        }
    };

    Ok(unsafe { RKEY.lock().await.get(&flag).cloned() })
}