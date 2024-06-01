use std::sync::Arc;
use anyhow::Error;
use bytes::{Buf, Bytes};
use jcers::Jce;
use log::error;
use ntrim_macros::servlet;
use ntrim_tools::tokiort::global_tokio_runtime;
use crate::bot::Bot;
use crate::client::packet::FromServiceMsg;
use crate::jce;
use crate::jce::onlinepush::reqpushmsg::PushMessageInfo;

pub struct OlPushServlet(Arc<Bot>);

#[servlet("trpc.msg.olpush.OlPushService.MsgPush", "OnlinePush.ReqPush")]
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
                //let bot = Arc::clone(&servlet.0);
                tokio::spawn(async move {
                    let _ = OlPushServlet::on_req_push(from).await.map_err(|e| {
                        error!("OlPushServlet::on_req_push error: {:?}", e);
                    });
                });
            },
            _ => {}
        };
    }

    async fn on_msg_push(bot: Arc<Bot>, from: FromServiceMsg) {

    }

    async fn on_req_push(mut from: FromServiceMsg) -> Result<(), Error> {
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


        Ok(())
    }
}