use std::sync::Arc;
use log::info;
use ntrim_macros::servlet;
use crate::bot::Bot;
use crate::client::packet::FromServiceMsg;

pub struct SyncPushServlet(Arc<Bot>);

#[servlet("trpc.msg.register_proxy.RegisterProxy.InfoSyncPush", "trpc.msg.register_proxy.RegisterProxy.PushParams")]
impl SyncPushServlet {
    async fn dispatch(servlet: &SyncPushServlet, from: FromServiceMsg) {


    }


}


