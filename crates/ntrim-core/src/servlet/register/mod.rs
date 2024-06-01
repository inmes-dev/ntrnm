use std::cell::OnceCell;
use std::sync::{Arc, OnceLock};
use chrono::DateTime;
use log::{info, warn};
use prost::{DecodeError, Message};
use ntrim_macros::servlet;
use crate::bot::Bot;
use crate::client::packet::FromServiceMsg;
use crate::db;
use crate::db::SimpleMessageRecord;
use crate::pb::trpc::register::{ * };

pub struct RegisterProxyServlet(Arc<Bot>);

#[servlet("trpc.msg.register_proxy.RegisterProxy.InfoSyncPush", "trpc.msg.register_proxy.RegisterProxy.PushParams")]
impl RegisterProxyServlet {
    async fn dispatch(servlet: &RegisterProxyServlet, from: FromServiceMsg) {
        match from.command.as_str() {
            "trpc.msg.register_proxy.RegisterProxy.PushParams" => Self::on_push_params(servlet, TrpcPushParams::decode(from.wup_buffer.as_slice())),
            "trpc.msg.register_proxy.RegisterProxy.InfoSyncPush" => Self::on_sync_push(servlet, TrpcInfoSyncPush::decode(from.wup_buffer.as_slice())).await,
            _ => {
                //info!("CMD: {}, PACKET: {}", from.command, hex::encode(from.wup_buffer.as_ref()));
            }
        }
    }

    async fn on_sync_push(servlet: &RegisterProxyServlet, sync_push: Result<TrpcInfoSyncPush, DecodeError>) {
        let sync_push = sync_push.unwrap();
        if sync_push.result.map_or(false, |v| v != 0) {
            return;
        }
        #[cfg(feature = "sql")]
        if sync_push.push_flag == 5 {
            let pool = db::PG_POOL.get().unwrap();
            for node in sync_push.group_nodes {
                let record = SimpleMessageRecord {
                    id: node.peer_id as i64,
                    seq: node.msg_seq as i64,
                    last_seq: node.longest_msg_seq.map_or(0, |v| v as i64),
                    name: node.peer_name.clone(),
                    latest_msg_time: DateTime::from_timestamp(node.latest_msg_time as i64, 0).unwrap().naive_utc(),
                };
                SimpleMessageRecord::insert(pool, record).await.map_err(|e| {
                    warn!("Failed to insert group_simple_record to pgsql: {:?}", e);
                }).unwrap();
            }
        }
    }

    fn on_push_params(servlet: &RegisterProxyServlet, push_params: Result<TrpcPushParams, DecodeError>) {
        static ENABLE_PRINT_PUSHPARAMS: OnceLock<bool> = OnceLock::new();
        if !*ENABLE_PRINT_PUSHPARAMS.get_or_init(|| option_env!("ENABLE_PRINT_PUSHPARAMS").map_or(false, |v| v == "1")) {
            return;
        }
        let push_params = push_params.unwrap();
        info!("Current online device count: {}", push_params.online_devices.len());
        let online_devices = push_params.online_devices;
        for i in 0..online_devices.len() {
            let device = &online_devices[i];
            info!("OnlineDevice[{}] app_id: {:?}, plat: {:?}, name: {:?}", i, device.inst_id, device.plat_type, device.device_name)
        }
        info!("Guild flags: {:?}", push_params.guild_params) // 频道权限
    }
}


