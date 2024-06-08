use std::sync::atomic::Ordering::SeqCst;
use std::time::Duration;
use chrono::Local;
use log::info;
use prost::Message;
use tokio::time::{Instant, interval};
use ntrim_macros::command;
use crate::bot::{BotStatus};
use crate::pb::trpc::status::{ * };

struct NtHeartBeatBuilder;

#[command("trpc.qq_new_tech.status_svc.StatusService.SsoHeartBeat", "send_nt_heartbeat", Protobuf, Service)]
impl NtHeartBeatBuilder {
    async fn generate(bot: &Arc<Bot>) -> Option<Vec<u8>> {
        let current_time = Local::now().timestamp();

        let level = 83;
        let charging = 1;

        let mut req = SsoHeartBeatRequest::default();
        req.r#type = 1;
        req.battery_state = (req.battery_state & !0x7F) | (level as u32 & 0x7F);
        req.battery_state = (req.battery_state & !(1 << 7)) | ((charging as u32 & 0x01) << 7);
        req.time = current_time as u64;
        req.local_silence = SilenceState { local_silence: 1 };
        Some(req.encode_to_vec())
    }

    async fn parse(bot: &Arc<Bot>, data: Vec<u8>) -> Option<u64> {
        let resp = SsoHeartBeatResponse::decode(&data[..]).ok()?;
        if let Some(interval) = resp.interval {
            return Some(interval)
        }
        None
    }
}