mod msg;
mod notice;

use std::collections::HashMap;
use std::sync::Arc;
use anyhow::Error;
use bytes::{Buf, Bytes};
use jcers::{Jce, JcePut};
use log::{error, warn};
use prost::Message;
use ntrim_macros::servlet;
use ntrim_tools::tokiort::global_tokio_runtime;
use crate::bot::Bot;
use crate::client::packet::FromServiceMsg;
use crate::client::packet::packet::UniPacket;
use crate::jce;
use crate::jce::onlinepush::reqpushmsg::{DelMsgInfo, PushMessageInfo, SvcRespPushMsg};
use crate::jce::pack_uni_request_data;
use crate::pb::trpc::olpush::MsgPush;

pub struct OlPushServlet(Arc<Bot>);

#[servlet(
    "trpc.msg.olpush.OlPushService.MsgPush",
    "OnlinePush.ReqPush"
)]
impl OlPushServlet {
    async fn dispatch(servlet: &OlPushServlet, from: FromServiceMsg) {
        match from.command.as_str() {
            "trpc.msg.olpush.OlPushService.MsgPush" => {
                let bot = Arc::clone(&servlet.0);
                tokio::spawn(async move {
                    let _ = OlPushServlet::on_msg_push(bot, from).await.map_err(|e| {
                        error!("OlPushServlet::on_msg_push error: {:?}", e);
                    });
                });
            },
            "OnlinePush.ReqPush" => {
                let bot = Arc::clone(&servlet.0);
                tokio::spawn(async move {
                    let _ = OlPushServlet::on_req_push(bot, from).await.map_err(|e| {
                        error!("OlPushServlet::on_req_push error: {:?}", e);
                    });
                });
            },
            _ => {}
        };
    }

    async fn on_msg_push(bot: Arc<Bot>, mut from: FromServiceMsg) -> Result<(), Error> {
        let msg = MsgPush::decode(Bytes::from(from.wup_buffer.clone()))?.msg;
        match msg.content_head.msg_type {
            //33 => notice::on_group_member_increase(bot, msg_push),
            //38 => notice::on_group_create(bot, msg_push),

            82 => msg::on_group_msg(bot, msg),
            //84 => notice::on_group_join_request(bot, msg_push),
            //85 => notice::on_group_join_request_approved(bot, msg_push),
            //87 => notice::on_group_invite(bot, msg_push),

            //141 => msg::on_stranger_msg(bot, msg_push),
            //166 => msg::on_friend_msg(bot, msg_push),
            //167 => msg::on_unidirectional_friend_msg(bot, msg_push),
            //187 => notice::on_friend_request_add(bot, msg_push),
            //191 => notice::on_unidirectional_friend_increase(bot, msg_push),
            //208 => msg::on_friend_audio_msg(bot, msg_push),

            //525 => notice::on_group_member_invite(bot, msg_push),
            //529 => notice::on_offline_file(bot, msg_push),

            _ => if option_env!("ENABLE_PRINT_UNKNOWN_PUSH").map_or(true, |v| v.parse::<bool>().unwrap()) {
                warn!("Unknown msg type: {:?}, buf: {}", msg.content_head.msg_type, hex::encode(&from.wup_buffer))
            }
        }
        Ok(())
    }

    /// 自动推送消息通知
    async fn on_req_push(bot: Arc<Bot>, mut from: FromServiceMsg) -> Result<(), Error> {
        let mut payload = Bytes::from(from.wup_buffer);
        let mut request: jce::RequestPacket = jcers::from_buf(&mut payload)?;
        let mut data: jce::RequestDataVersion2 = jcers::from_buf(&mut request.s_buffer)?;
        let mut req = data
            .map
            .remove("req")
            .ok_or_else(|| Error::msg("req is none"))?;
        let mut msg = req
            .remove("OnlinePushPack.SvcReqPushMsg")
            .ok_or_else(|| Error::msg("OnlinePushPack.SvcReqPushMsg is none"))?;
        msg.advance(1);
        let mut jr = Jce::new(&mut msg);
        let uin = jr.get_by_tag::<i64>(0)?;
        let msg_infos: Vec<PushMessageInfo> = jr.get_by_tag(2)?;

        let req = SvcRespPushMsg {
            uin,
            svrip: 0,
            push_token: Bytes::new(),
            del_infos: msg_infos
                .into_iter()
                .map(|m| DelMsgInfo {
                    from_uin: m.from_uin,
                    msg_time: m.msg_time,
                    msg_seq: m.msg_seq,
                    msg_cookies: m.msg_cookies,
                    ..Default::default()
                })
                .collect(),
            ..Default::default()
        };
        let b = pack_uni_request_data(&req.freeze());
        let buf = jce::RequestDataVersion3 {
            map: HashMap::from([("resp".to_string(), b)]),
        };
        let pkt = jce::RequestPacket {
            i_version: 2,
            i_request_id: request.i_request_id,
            s_servant_name: "OnlinePush".to_string(),
            s_func_name: "SvcRespPushMsg".to_string(),
            s_buffer: buf.freeze(),
            ..Default::default()
        };
        let payload = pkt.freeze().to_vec();
        let pkt = UniPacket::new_service("OnlinePush.RespPush".into(), payload);
        let _ = bot.client.send_uni_packet(pkt).await;
        Ok(())
    }
}