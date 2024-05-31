pub mod register;
pub mod register_old;
pub mod wtlogin;
mod status;
mod richmedia;

/// timeout不可以小于5s时间，否则可能导致内存泄露
#[macro_export]
macro_rules! await_response {
    ($timeout:expr, $future:expr, $success_handler:expr, $error_handler:expr) => {
        match tokio::time::timeout($timeout, $future).await {
            Ok(result) => match result {
                Ok(value) => $success_handler(value),
                Err(e) => $error_handler(e),
            },
            Err(_) => $error_handler(anyhow::Error::msg("Timeout occurred")),
        }
    };
}

#[macro_export]
macro_rules! oidb_request {
    ($cmd:expr, $service:expr, $buffer:expr) => {
         Some(pb::oidb::TrpcOidbRequest {
            cmd: $cmd,
            service: $service,
            body: $buffer,
            nt_flag: Some(12),
        }.encode_to_vec())
    };
}

#[macro_export]
macro_rules! oidb_response {
    ($cmd:expr, $service:expr, $buffer:expr) => {{
        let data = match pb::oidb::TrpcOidbResponse::decode($buffer) {
            Ok(data) => Some(data),
            Err(e) => {
                info!("Failed to decode TrpcOidbResponse: {:?}", e);
                None
            }
        };
        if let Some(rsp) = data {
            if rsp.cmd != $cmd || rsp.service != $service {
                info!("Invalid TrpcOidbResponse: {:?}", rsp);
                None
            } else {
                Some(rsp.body)
            }
        } else {
            info!("Invalid TrpcOidbResponse: {:?}", $buffer);
            None
        }
    }};
}