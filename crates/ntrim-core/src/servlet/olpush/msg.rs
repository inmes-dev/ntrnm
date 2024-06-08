use std::sync::Arc;
use log::{info, warn};
use crate::bot::Bot;
use crate::pb::trpc::olpush::{*};

pub(super) fn on_group_msg(bot: Arc<Bot>, msg: Message) {
    let msg_time = msg.content_head.msg_time;
    let msg_seq = msg.content_head.msg_seq;
    let msg_uid = msg.content_head.msg_uid;
    let (sender_uid, sender_uin) = (msg.routing_head.peer_uid.unwrap(), msg.routing_head.peer_id);
    let from_sub_appid = msg.routing_head.from_app_id;
    let platform = msg.routing_head.platform;
    let (group_id, sender_nick, group_name) = match msg.routing_head.contact {
        Some(routing_head::Contact::Grp(grp)) => (
            grp.group_id,
            grp.sender_nick.map_or_else(|| "".to_string(), |x| x),
            grp.group_name.map_or_else(|| "".to_string(), |x| x)
        ),
        _ => {
            warn!("Invalid routing_head, msg_seq: {}", msg_seq);
            return;
        }
    };

    if msg.msg_body.rich_text.is_none() {
        warn!("Empty rich_text, msg_seq: {}", msg_seq);
        return;
    }
    let mut rich_text = msg.msg_body.rich_text.unwrap();
    let cq_code = decoder::parse_elements(rich_text.elems);

    println!("群消息 [{}({})] {}({}): {}", group_name, group_id, sender_nick, sender_uin,
        cq_code.iter().map(|x| x.to_string()).collect::<Vec<String>>().join("")
    );
}

mod decoder {
    use bytes::{Buf, Bytes};
    use log::warn;
    pub use ntrim_tools::cqp::CQCode;
    use crate::pb::trpc::olpush::{ * };
    use crate::pb::trpc::olpush::elem::{*};

    pub(super) fn parse_elements(elems: Vec<Elem>) -> Vec<CQCode> {
        let mut single_element = false;
        let mut result = Vec::new();
        for elem in elems {
            if elem.aio_elem.is_none() {
                warn!("Unsupported elem found, skip this!");
                continue;
            }
            let elem = elem.aio_elem.unwrap();
            match elem {
                AioElem::Text(Text { text, attr_6, .. }) => {
                    if attr_6.is_some() {
                        let mut buf = Bytes::from(attr_6.unwrap());
                        let size = buf.get_u16();
                        let pos = buf.get_u16();
                        let nick_len = buf.get_u16();
                        let is_at_all = buf.get_u8();
                        let uin = buf.get_u32() as u64;
                        result.push(CQCode::Special {
                            cq_type: "at".to_string(),
                            params: vec![
                                ("qq".to_string(), uin.to_string()),
                                #[cfg(feature = "extend_cqcode")]
                                ("content".to_string(), text.clone()),
                            ].into_iter().collect(),
                        })
                    } else {
                        result.push(CQCode::Text(text));
                    }
                }

                AioElem::ArkJson(LightArk { data }) => {
                    warn!("Unsupported ArkJson")
                }

                AioElem::CommonElem(CommonElem{ service_type, data, business_type }) => {
                    warn!("Unsupported CommonElem")
                }
            }
        }
        result
    }
}

mod encoder {

}