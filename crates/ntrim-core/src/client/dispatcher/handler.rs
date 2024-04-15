use rc_box::ArcBox;
use crate::client::packet::from_service_msg::FromServiceMsg;

pub trait TrpcHandler {
    fn handle(arc_self: ArcBox<Self>, msg: &FromServiceMsg);
}
