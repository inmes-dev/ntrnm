use std::sync::Arc;
use crate::client::Client;
use crate::codec::qqsecurity::QSecurity;

impl QSecurity for Client {
    fn energy(&self, data: String, salt: Box<[u8]>) -> Vec<u8> {
        todo!()
    }

    fn sign(&self, uin: String, cmd: String, buffer: Arc<Vec<u8>>, seq: u32) -> crate::codec::qqsecurity::QSecurityResult {
        todo!()
    }
}