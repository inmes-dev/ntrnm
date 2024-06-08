use std::ops::Deref;
use std::process::exit;
use chrono::Local;
use rand::{Rng, thread_rng};
use rand::distributions::Alphanumeric;
use ntrim_core::session::device::Device;
use ntrim_core::session::protocol::protocol;
use ntrim_core::session::SsoSession;
use ntrim_core::session::ticket::{SigType, Ticket, TicketManager};
use ntrim_tools::crypto::qqtea::qqtea_decrypt;

/// 为宿主生成随机社会唯一身份ID
fn rand_qimei() -> String {
    use rand::Rng;
    const CHARSET: &[u8] = b"abcdef0123456789";
    const LEN: usize = 36;
    let mut rng = thread_rng();
    (0..LEN).map(|_| {
        let idx = rng.gen_range(0..CHARSET.len());
        CHARSET[idx] as char
    }).collect()
}

/// 保存克隆体
pub fn save_session(path: &str, session: &SsoSession) {
    info!("Saving session to {}", path);
    let mut data = serde_json::Map::new();
    /// 仿生环境保存
    data.insert("uin".to_string(), serde_json::Value::String(session.uin.to_string()));
    data.insert("uid".to_string(), serde_json::Value::String(session.uid.clone()));
    data.insert("ksid".to_string(), serde_json::Value::String(hex::encode(session.ksid)));
    data.insert("guid".to_string(), serde_json::Value::String(hex::encode(session.guid)));
    let device = &session.device;
    data.insert("android_id".to_string(), serde_json::Value::String(device.android_id.to_string()));
    data.insert("dev_name".to_string(), serde_json::Value::String(device.device_name.to_string()));
    data.insert("os_ver".to_string(), serde_json::Value::String(device.os_ver.to_string()));
    data.insert("code".to_string(), serde_json::Value::String(device.code.to_string()));
    data.insert("os_name".to_string(), serde_json::Value::String(device.os_name.to_string()));
    data.insert("fingerprint".to_string(), serde_json::Value::String(hex::encode(device.fingerprint.as_ref())));
    data.insert("brand".to_string(), serde_json::Value::String(device.brand.to_string()));
    data.insert("vendor_os_name".to_string(), serde_json::Value::String(device.vendor_os_name.to_string()));

    /// RNA冷冻保存
    let mut ticket = serde_json::Map::new();
    for (id, t) in &session.tickets {
        let mut ticket_data = serde_json::Map::new();
        if let Some(sig) = &(t.sig) {
            if sig.is_empty() { continue }
            ticket_data.insert("sig".to_string(), serde_json::Value::String(hex::encode(sig.as_slice())));
        } else {
            ticket_data.insert("sig".to_string(), serde_json::Value::String("".to_string()));
        }
        ticket_data.insert("sigKey".to_string(), serde_json::Value::String(hex::encode(t.sig_key.as_slice())));
        ticket_data.insert("createTime".to_string(), serde_json::Value::Number(serde_json::Number::from(t.create_time)));
        ticket_data.insert("expireTime".to_string(), serde_json::Value::Number(serde_json::Number::from(t.expire_time)));
        ticket.insert(id.bits().to_string(), serde_json::Value::Object(ticket_data));
    }
    data.insert("ticket".to_string(), serde_json::Value::Object(ticket));

    /// 冷冻保存DNA
    let mut sigs = serde_json::Map::new();
    let en_a1 = [&session.encrypt_a1[..], &session.tgtgt_key[..]].concat();
    sigs.insert("en_a1".to_string(), serde_json::Value::String(hex::encode(en_a1)));
    sigs.insert("no_pic_sig".to_string(), serde_json::Value::String(hex::encode(&session.no_pic_sig)));
    sigs.insert("wt_session_ticket".to_string(), serde_json::Value::String(hex::encode(&session.wt_session_ticket)));
    sigs.insert("wt_session_key".to_string(), serde_json::Value::String(hex::encode(&session.wt_session_key)));
    sigs.insert("wt_session_create_time".to_string(), serde_json::Value::Number(serde_json::Number::from(session.wt_session_create_time)));
    data.insert("sigs".to_string(), serde_json::Value::Object(sigs));

    /// 记录黑盒最后时间
    data.insert("update_time".to_string(), serde_json::Value::String(Local::now().to_rfc3339()));

    let data = serde_json::Value::Object(data);
    std::fs::write(path, serde_json::to_string_pretty(&data).unwrap()).unwrap();
}

fn is_valid_en_a1(en_a1: &[u8]) -> bool {
    en_a1.iter().all(|&x| x >= 33 && x <= 126)
}

/// 载入克隆体
pub fn load_session(path: &str) -> SsoSession {
    let current_sec_time = Local::now().timestamp();
    info!("Loading cache session from {}", path);
    let data = std::fs::read_to_string(path).unwrap();
    let session_data: serde_json::Value = serde_json::from_str(&data).unwrap();
    let session_data = session_data.as_object().unwrap();

    /// 仿生环境
    let uin = session_data["uin"].as_str().unwrap();
    info!("Loaded session for uin: {}", uin);
    let uid = session_data["uid"].as_str().unwrap();
    let ksid = hex::decode(session_data["ksid"].as_str().unwrap()).unwrap();
    let guid = hex::decode(session_data["guid"].as_str().unwrap()).unwrap();
    let mut android_id = session_data["android_id"].as_str().unwrap();
    if android_id.len() > 16 {
        warn!("Android id is too long, maybe it's not a valid android id: {}", android_id);
        android_id = &android_id[..16];
        warn!("Truncated android id to: {}", android_id)
    } else if android_id.len() < 16 {
        error!("Android id is too short, maybe it's not a valid android id: {}", android_id);
        exit(1);
    }
    let dev_name = session_data["dev_name"].as_str().unwrap();
    let os_ver = session_data["os_ver"].as_str().unwrap();
    let code = session_data["code"].as_str().unwrap();
    let os_name = session_data["os_name"].as_str().unwrap();
    let fingerprint = hex::decode(session_data["fingerprint"].as_str().unwrap()).unwrap();
    let brand = session_data["brand"].as_str().unwrap();
    let vendor_os_name = session_data["vendor_os_name"].as_str().unwrap();
    let ticket = session_data["ticket"].as_object().unwrap();
    let device = Device::new(
        android_id.to_string(),
        rand_qimei(),
        dev_name.to_string(),
        brand.to_string(),
        os_ver.to_string(),
        vendor_os_name.to_string(),
        fingerprint,
        code.to_string(),
        os_name.to_string()
    );

    /// 遗传信息
    let protocol = protocol::qq_9_0_20();
    let mut sso_session = SsoSession::new(
        (uin.parse().unwrap(), uid.to_string()),
        protocol.clone(),
        device,
        ksid.as_slice().try_into().unwrap(),
        guid.as_slice().try_into().unwrap()
    );

    /// DNA的复制
    let sigs = session_data["sigs"].as_object().unwrap();
    let mut a1_with_tgtgt_key = hex::decode(sigs["en_a1"].as_str().unwrap()).unwrap();
    // 解码管家基因
    if is_valid_en_a1(&a1_with_tgtgt_key[a1_with_tgtgt_key.len() - 16..]) {
        sso_session.encrypt_a1 = a1_with_tgtgt_key[..a1_with_tgtgt_key.len() - 16].to_vec();
        sso_session.tgtgt_key = a1_with_tgtgt_key[a1_with_tgtgt_key.len() - 16..].to_vec();
    } else {
        warn!("Your A1 is invalid, try to decrypt with guid");
        a1_with_tgtgt_key = qqtea_decrypt(a1_with_tgtgt_key.as_slice(), guid.as_slice()).unwrap();
        sso_session.encrypt_a1 = a1_with_tgtgt_key[..a1_with_tgtgt_key.len() - 16].to_vec();
        sso_session.tgtgt_key = a1_with_tgtgt_key[a1_with_tgtgt_key.len() - 16..].to_vec();
    }
    sso_session.no_pic_sig = hex::decode(sigs["no_pic_sig"].as_str().unwrap()).unwrap();
    sso_session.wt_session_ticket = hex::decode(sigs["wt_session_ticket"].as_str().unwrap()).unwrap();
    sso_session.wt_session_key = hex::decode(sigs["wt_session_key"].as_str().unwrap()).unwrap();
    sso_session.wt_session_create_time = sigs["wt_session_create_time"].as_u64().unwrap();
    let wt_session_expire_time = current_sec_time as u64 - sso_session.wt_session_create_time;
    if wt_session_expire_time <= 2592000 {
        warn!("WT session expired, please refresh st: cur_time: {}, create_time: {}", current_sec_time, sso_session.wt_session_create_time);
    } else {
        info!("WT session expire in {} seconds", 2592000 - wt_session_expire_time);
    }

    /// RNA复制
    /// WARN：该区域存在大量没有科学价值的RNA
    for (k, v) in ticket {
        let sig_type: u32 = k.parse().unwrap();
        let sig_type = SigType::from_bits(sig_type).unwrap();
        let v = v.as_object().unwrap();
        let sig = hex::decode(v["sig"].as_str().unwrap()).unwrap();
        let key = hex::decode(v["sigKey"].as_str().unwrap()).unwrap();
        let create_time = v["createTime"].as_i64().unwrap();
        let expire_time = v["expireTime"].as_i64().unwrap();
        sso_session.insert(Ticket {
            id: sig_type,
            sig_key: key,
            sig: Some(sig),
            create_time,
            expire_time,
        });
    }
    return sso_session;
}