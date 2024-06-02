use std::collections::HashMap;
use std::sync::Arc;
use anyhow::Error;
use bytes::{Buf, Bytes};
use jcers::{Jce, JcePut};
use log::error;
use ntrim_macros::servlet;
use ntrim_tools::tokiort::global_tokio_runtime;
use crate::bot::Bot;
use crate::client::packet::FromServiceMsg;
use crate::client::packet::packet::UniPacket;
use crate::jce;
use crate::jce::onlinepush::reqpushmsg::{DelMsgInfo, PushMessageInfo, SvcRespPushMsg};
use crate::jce::pack_uni_request_data;

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
                    OlPushServlet::on_msg_push(bot, from).await;
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

    async fn on_msg_push(bot: Arc<Bot>, from: FromServiceMsg) {

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