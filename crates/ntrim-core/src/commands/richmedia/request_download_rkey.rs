
use log::info;
use prost::{DecodeError, Message};
use sqlx::encode::IsNull::No;
use ntrim_macros::command;
use crate::{oidb_request, oidb_response, pb};
use crate::pb::trpc::rich_media_ntv2::{ * };

struct RequestDownloadRKeyBuilder;

#[command("OidbSvcTrpcTcp.0x9067_202", "request_download_rkey", Protobuf, Service)]
impl RequestDownloadRKeyBuilder {
    async fn generate(bot: &Arc<Bot>) -> Option<Vec<u8>> {
        oidb_request!(0x9067, 202, NtV2RichMediaReq {
            head: MultiMediaReqHead {
                head: CommonHead {
                    req_id: 1,
                    cmd: 202,
                    msg: None,
                },
                scene: SceneInfo {
                    request_type: 2,
                    business_type: 1,
                    app_type: None,
                    scene_type: Some(0),
                    c2c: None, grp: None, channel: None, byte_arr: None,
                },
                client_meta: ClientMeta {
                    agent_type: 2,
                },
            },
            download: Some(DownloadRkeyReq {
                types: vec![10, 20],
                download_type: Some(2),
            }),
        }.encode_to_vec())
    }

    async fn parse(bot: &Arc<Bot>, data: Vec<u8>) -> Option<DownloadRkeyRsp> {
        let response = oidb_response!(0x9067, 202, data.as_slice())?;
        match NtV2RichMediaRsp::decode(response.as_slice()) {
            Ok(v) => {
                if v.head.ret_code.is_none() || v.head.ret_code == Some(0) {
                    Some(v.download_rkey_rsp.unwrap())
                } else {
                    error!("Failed to request download rkey, code: {:?}, msg: {}", v.head.ret_code, v.head.msg);
                    None
                }
            },
            Err(e) => {
                error!("Failed to decode NtV2RichMediaRsp(202): {:?}, data: {}", e, hex::encode(&response));
                None
            }
        }
    }
}

