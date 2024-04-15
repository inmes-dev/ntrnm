use rc_box::ArcBox;
use crate::client::dispatcher::handler::TrpcHandler;
use crate::client::packet::from_service_msg::FromServiceMsg;

pub(crate) struct PersistentHandler {
    command: String,
    handler: Box<dyn Fn(FromServiceMsg) + Send + Sync>
}

pub fn new(
    command: String,
    handler: Box<dyn Fn(FromServiceMsg) + Send + Sync>
) -> ArcBox<PersistentHandler> {
    ArcBox::new(PersistentHandler {
        command,
        handler
    })
}

impl TrpcHandler for PersistentHandler {
    fn handle(arc_self: ArcBox<Self>, msg: &FromServiceMsg) {
        if &arc_self.command == &msg.command {
            (arc_self.handler)(msg.clone());
        }
    }
}