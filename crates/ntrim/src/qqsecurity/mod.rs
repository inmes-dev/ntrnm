use std::env;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use reqwest::Client;
use ntrim_core::client::qsecurity::{QSecurity, QSecurityResult};

#[derive(Debug)]
pub(crate) struct QSecurityViaHTTP {
    pub(crate) sign_server: String,
    pub(crate) client: Client
}

fn get_system_proxy() -> Option<String> {
    if cfg!(target_os = "windows") {
        env::var("HTTPS_PROXY").ok()
            .or_else(|| env::var("https_proxy").ok())
            .or_else(|| env::var("HTTP_PROXY").ok())
            .or_else(|| env::var("http_proxy").ok())
    } else if cfg!(target_os = "macos") || cfg!(target_os = "linux") {
        env::var("https_proxy").ok()
            .or_else(|| env::var("HTTPS_PROXY").ok())
            .or_else(|| env::var("http_proxy").ok())
            .or_else(|| env::var("HTTP_PROXY").ok())
    } else {
        None
    }
}

impl QSecurityViaHTTP {
    pub fn new(sign_server: &str) -> Self {
        let sign_server = if sign_server.ends_with("/") {
            sign_server.to_string()
        } else {
            format!("{}/", sign_server)
        };
        let mut builder = Client::builder()
            .connect_timeout(std::time::Duration::from_secs(15))
            .timeout(std::time::Duration::from_secs(30))
            .tcp_nodelay(true)
            .use_rustls_tls();

        let proxy_url = get_system_proxy();
        if let Some(proxy_url) = proxy_url {
            match reqwest::Proxy::all(&proxy_url) {
                Ok(proxy) => {
                    builder = builder.proxy(proxy);
                }
                Err(e) => {
                    warn!("Failed to set proxy: {}", e)
                }
            };
        }

        Self {
            sign_server,
            client: builder.build().unwrap()
        }
    }
}

impl QSecurity for QSecurityViaHTTP {
    fn ping<'a>(&'a self) -> Pin<Box<dyn Future<Output=bool> + Send + 'a>> {
        Pin::from(Box::new(async move {
            let response = match self.client
                .get(self.sign_server.clone() + "ping")
                .send()
                .await {
                Ok(response) => response,
                Err(e) => {
                    error!("Failed to ping sign server (0x0): {}", e);
                    return false;
                }
            };
            let response = response.text().await.map_err(|e| {
                return false;
            }).unwrap();
            let response: serde_json::Value = serde_json::from_str(&response).map_err(|e| {
                error!("Failed to ping sign server (0x2): {}, json: {}", e, response);
                return false;
            }).unwrap();
            let ret = response["retcode"].as_u64().unwrap();
            if ret != 0 {
                let msg = response["message"].as_str().unwrap();
                error!("Failed to get ping response ret: {}, msg: {}", ret, msg);
                return false;
            }
            let data = response["data"].as_object().unwrap();
            let qua = data["qua"].as_str().unwrap();
            debug!("Ping sign server success, qua: {}", qua);
            return true;
        }))
    }

    fn energy<'a>(&'a self, data: String, salt: Vec<u8>) -> Pin<Box<dyn Future<Output=Vec<u8>> + Send + 'a>> {
        Pin::from(Box::new(async move {
            let start = std::time::Instant::now();
            let salt = hex::encode(salt.as_slice());
            let params = [("data", data), ("salt", salt)];
            let response = self.client
                .post(self.sign_server.clone() + "custom_energy")
                .form(&params)
                .send()
                .await
                .unwrap();
            let response = response.text().await.unwrap();
            let response: serde_json::Value = serde_json::from_str(&response).unwrap();
            let ret = response["retcode"].as_u64().unwrap_or(1);
            if ret != 0 {
                let msg = response["message"].as_str().unwrap_or_else(|| "Unknown error");
                log::error!("Failed to get custom_energy response ret: {}, msg: {}", ret, msg);
                return vec![];
            }
            let cost_time = start.elapsed().as_millis();
            info!("Energy request cost: {}ms", cost_time);
            let data = response["data"].as_object().unwrap();
            let data = data["data"].as_str().unwrap();
            return hex::decode(data).unwrap();
        }))
    }

    fn sign<'a>(&'a self, uin: String, cmd: String, buffer: Arc<Vec<u8>>, seq: u32) -> Pin<Box<dyn Future<Output=QSecurityResult> + Send + 'a>> {
        Pin::from(Box::new(async move {
            let start = std::time::Instant::now();
            let buffer = hex::encode(buffer.as_slice());
            let params = [
                ("uin", uin),
                ("cmd", cmd),
                ("seq", seq.to_string()),
                ("buffer", buffer)
            ];
            let response = self.client.post(self.sign_server.clone() + "sign").form(&params).send().await.map_err(|e| {
                error!("Failed to send sign request: {}", e);
                return QSecurityResult::new_empty();
            }).unwrap();
            let response = response.text().await.map_err(|e| {
                log::error!("Failed to get sign response text: {}", e);
                return QSecurityResult::new_empty();
            }).unwrap();
            let response: serde_json::Value = serde_json::from_str(&response).unwrap();
            let ret = response["retcode"].as_u64().unwrap_or(1);
            if ret != 0 {
                let msg = response["message"].as_str().unwrap_or_else(|| "Unknown error");
                log::error!("Failed to get sign response ret: {}, msg: {}", ret, msg);
                return QSecurityResult::new_empty();
            }
            let cost_time = start.elapsed().as_millis();
            info!("Sign request cost: {}ms", cost_time);
            let data = response["data"].as_object().unwrap();
            let sign = data["sign"].as_str().unwrap();
            let token = data["token"].as_str().unwrap();
            let extra = data["extra"].as_str().unwrap();
            let sign = hex::decode(sign).unwrap();
            let token = hex::decode(token).unwrap();
            let extra = hex::decode(extra).unwrap();
            let sign = Box::new(sign);
            let token = Box::new(token);
            let extra = Box::new(extra);
            QSecurityResult::new(sign, token, extra)
        }))
    }
}