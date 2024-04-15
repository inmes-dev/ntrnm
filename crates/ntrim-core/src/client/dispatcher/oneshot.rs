use std::ops::DerefMut;
use rc_box::ArcBox;
use crate::client::dispatcher::handler::TrpcHandler;
use crate::client::packet::from_service_msg::FromServiceMsg;

pub(crate) struct OneshotHandler {
    seq: u32,
    is_final: bool,
    handler: Box<dyn Fn(FromServiceMsg) + Send + Sync>
}

pub fn new(
    seq: u32,
    handler: Box<dyn Fn(FromServiceMsg) + Send + Sync>
) -> ArcBox<OneshotHandler> {
    ArcBox::new(OneshotHandler {
        seq,
        is_final: false,
        handler
    })
}

impl TrpcHandler for OneshotHandler {
    fn handle(mut arc_self: ArcBox<Self>, msg: &FromServiceMsg) {
        if arc_self.is_final {
            return;
        }
        if arc_self.seq == msg.seq {
            let a = arc_self.deref_mut();
            (arc_self.handler)(msg.clone());
        }
    }
}