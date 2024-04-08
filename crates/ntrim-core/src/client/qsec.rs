use std::sync::Arc;

use crate::client::Client;
use crate::codec::qqsecurity::{QSecurity, QSecurityResult};

impl QSecurity for Client {
    async fn energy(&self, data: String, salt: Box<[u8]>) -> Vec<u8> {
        todo!()
    }

    async fn sign(&self, uin: String, cmd: String, buffer: Arc<Vec<u8>>, seq: u32) -> QSecurityResult {
        let client = reqwest::Client::new();
        let sign_server = self.session.sign_server.clone();
        let buffer = hex::encode(buffer.as_slice());
        let urlencoded = reqwest::multipart::Form::new()
            .text("uin", uin)
            .text("cmd", cmd)
            .text("seq", seq.to_string())
            .text("buffer", buffer);
        let response = match client.post(sign_server + "sign")
            .multipart(urlencoded)
            .send()
            .await {
            Ok(response) => response,
            Err(e) => {
                log::error!("Failed to send sign request: {}", e);
                return QSecurityResult::new_empty();
            }
        };
        let response = match response.text().await {
            Ok(response) => response,
            Err(e) => {
                log::error!("Failed to get sign response text: {}", e);
                return QSecurityResult::new_empty();
            }
        };
        let response: serde_json::Value = match serde_json::from_str(&response) {
            Ok(response) => response,
            Err(e) => {
                log::error!("Failed to parse sign response: {}", e);
                return QSecurityResult::new_empty();
            }
        };
        let ret = match response["retcode"].as_u64() {
            Some(ret) => ret as u32,
            None => {
                log::error!("Failed to get sign response ret: {:?}", response);
                return QSecurityResult::new_empty();
            }
        };
        if ret != 0 {
            let msg = match response["message"].as_str() {
                Some(msg) => msg,
                None => {
                    log::error!("Failed to get sign response msg: {:?}", response);
                    return QSecurityResult::new_empty();
                }
            };
            log::error!("Failed to get sign response ret: {}, msg: {}", ret, msg);
            return QSecurityResult::new_empty();
        }
        let data = match response["data"].as_object() {
            Some(data) => data,
            None => {
                log::error!("Failed to get sign response data: {:?}", response);
                return QSecurityResult::new_empty();
            }
        };
        let sign = match data["sign"].as_str() {
            Some(sign) => sign,
            None => {
                log::error!("Failed to get sign response sign: {:?}", response);
                return QSecurityResult::new_empty();
            }
        };
        let token = match data["token"].as_str() {
            Some(token) => token,
            None => {
                log::error!("Failed to get sign response token: {:?}", response);
                return QSecurityResult::new_empty();
            }
        };
        let extra = match data["extra"].as_str() {
            Some(extra) => extra,
            None => {
                log::error!("Failed to get sign response extra: {:?}", response);
                return QSecurityResult::new_empty();
            }
        };
        let sign = hex::decode(sign).unwrap();
        let token = hex::decode(token).unwrap();
        let extra = hex::decode(extra).unwrap();
        let sign = Box::new(sign);
        let token = Box::new(token);
        let extra = Box::new(extra);

        QSecurityResult::new(sign, token, extra)
    }
}