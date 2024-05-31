use std::sync::Arc;
use ntrim_macros::servlet;
use crate::bot::Bot;
use crate::client::packet::FromServiceMsg;

pub struct OlPushServlet(Arc<Bot>);

#[servlet("trpc.msg.olpush.OlPushService.MsgPush")]
impl OlPushServlet {
    async fn dispatch(servlet: &OlPushServlet, from: FromServiceMsg) {
        match from.command.as_str() {
            _ => {}
        }
    }
}