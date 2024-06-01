use jcers::{JceGet, JcePut};
use bytes::Bytes;

jce_struct!(PushMessageInfo {
    0 => from_uin: i64,
    1 => msg_time: i64,
    2 => msg_type: i16,
    3 => msg_seq: i16,
    4 => msg: String,
    5 => real_msg_time: i32,
    6 => v_msg: Bytes,
    7 => app_share_id: i64,
    8 => msg_cookies: Bytes,
    9 => app_share_cookie: Bytes,
    10 => msg_uid: i64,
    11 => last_change_time: i64,
    14 => from_inst_id: i64,
    15 => remark_of_sender: Bytes,
    16 => from_mobile: String,
    17 => from_name: String,
});

jce_struct!(SvcRespPushMsg {
    0 => uin: i64,
    1 => del_infos: Vec<DelMsgInfo>,
    2 => svrip: i32,
    3 => push_token: Bytes,
    4 => service_type: i32,
});

jce_struct!(DelMsgInfo {
    0 => from_uin: i64,
    1 => msg_time: i64,
    2 => msg_seq: i16,
    3 => msg_cookies: Bytes,
    4 => cmd: i16,
    5 => msg_type: i64,
    6 => app_id: i64,
    7 => send_time: i64,
    8 => sso_seq: i32,
    9 => sso_ip: i32,
    10 => client_ip: i32,
});