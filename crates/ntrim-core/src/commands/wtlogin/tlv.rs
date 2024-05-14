use std::fmt::format;
use std::time::UNIX_EPOCH;
use bytes::{BufMut, BytesMut};
use prost::Message;
use ntrim_tools::bytes::{BytePacketBuilder, PacketFlag};
use ntrim_tools::crypto::qqtea::qqtea_encrypt;
use crate::commands::wtlogin;

#[inline]
fn tlv_builder(buf: &mut BytesMut, ver: u16, body: &dyn Fn(&mut BytesMut)) {
    let mut body_buf = BytesMut::new();
    body(&mut body_buf);
    buf.put_u16(ver);
    buf.put_u16(body_buf.len() as u16);
    buf.put_slice(&body_buf);
}

pub fn t1(buf: &mut BytesMut, uin: u32) {
    tlv_builder(buf, 0x1, &|w| {
            w.put_u16(1);
            w.put_u32(rand::random());
            w.put_u32(uin);
            w.put_u32(UNIX_EPOCH.elapsed().unwrap().as_secs() as u32);
            //w.put_slice(ip);
            w.put_u32(0); // fake ip
            w.put_u16(0);
        },
    )
}

pub fn t8(buf: &mut BytesMut, locale_id: u32) {
    tlv_builder(buf, 0x8, &|w| {
            w.put_u16(0);
            w.put_u32(locale_id);
            w.put_u16(0);
        },
    )
}

pub fn t18(buf: &mut BytesMut, uin: u32) {
    tlv_builder(buf, 0x18, &|w| {
            w.put_u16(1);
            w.put_u32(1536);
            w.put_u32(16);
            w.put_u32(0);
            w.put_u32(uin);
            w.put_u16(0);
            w.put_u16(0);
        },
    )
}

pub fn t100(
    buf: &mut BytesMut,
    sso_version: u32,
    sub_app_id: u32,
    main_sig_map: u32,
    aid: u32,
) {
    tlv_builder(buf, 0x100, &|w| {
            w.put_u16(1);
            w.put_u32(sso_version);
            w.put_u32(aid);
            w.put_u32(sub_app_id);
            w.put_u32(0); // App client version
            w.put_u32(main_sig_map); // 34869472
        },
    )
}

pub fn t106_data(buf: &mut BytesMut, en_a1: &[u8]) {
    tlv_builder(buf, 0x106, &|w| {
        w.put_slice(en_a1);
    })
}

pub fn t107(buf: &mut BytesMut) {
    tlv_builder(buf, 0x107, &|w| {
        w.put_u16(0);
        w.put_u8(0x00);
        w.put_u16(0);
        w.put_u8(0x01);
    })
}

pub fn t108(buf: &mut BytesMut, ksid: &[u8]) {
    tlv_builder(buf, 0x108, &|w| {
        w.put_slice(ksid);
    })
}

pub fn t109(buf: &mut BytesMut, android_id: &str) {
    let md5 = md5::compute(android_id.as_bytes());
    tlv_builder(buf, 0x109, &|w| {
        w.put_slice(md5.as_slice());
    })
}

pub fn t116(buf: &mut BytesMut, misc_bitmap: u32, sub_sig_map: u32) {
    tlv_builder(buf, 0x116, &|w| {
            w.put_u8(0x00);
            w.put_u32(misc_bitmap); // 184024956
            w.put_u32(sub_sig_map); // 0x10400
            w.put_u8(0x01);
            w.put_u32(1600000226); // app id list
        },
    )
}

pub fn t124(
    buf: &mut BytesMut,
    os_type: &str,
    os_version: &str,
    sim_info: &str,
    apn: &str,
) {
    tlv_builder(buf, 0x124, &|w| {
            w.put_bytes_with_flags(os_type.as_bytes(), PacketFlag::I16Len);
            w.put_bytes_with_flags(os_version.as_bytes(), PacketFlag::I16Len);
            w.put_i16(2);
            w.put_bytes_with_flags(sim_info.as_bytes(), PacketFlag::I16Len);
            w.put_bytes_with_flags(apn.as_bytes(), PacketFlag::I32Len);
        },
    )
}

pub fn t128(
    buf: &mut BytesMut,
    is_guid_from_file_null: bool,
    is_guid_available: bool,
    is_guid_changed: bool,
    guid_flag: u32,
    build_model: &str,
    guid: &[u8],
    build_brand: &str,
) {
    tlv_builder(buf, 0x128, &|w| {
            w.put_u16(0);
            w.put_u8(if is_guid_from_file_null { 1 } else { 0 });
            w.put_u8(if is_guid_available { 1 } else { 0 });
            w.put_u8(if is_guid_changed { 1 } else { 0 });
            w.put_u32(guid_flag);
            w.put_bytes_with_flags(build_model.as_bytes(), PacketFlag::I16Len);
            w.put_bytes_with_flags(guid, PacketFlag::I16Len);
            w.put_bytes_with_flags(build_brand.as_bytes(), PacketFlag::I16Len); // app id list
        },
    )
}

pub fn t141(buf: &mut BytesMut, sim_info: &str, apn: &str) {
    tlv_builder(buf, 0x141, &|w| {
            w.put_u16(1);
            w.put_bytes_with_flags(sim_info.as_bytes(), PacketFlag::I16Len);
            w.put_u16(2);
            w.put_bytes_with_flags(apn.as_bytes(), PacketFlag::I16Len);
        },
    )
}

pub fn t142(
    buf: &mut BytesMut,
    apk_id: &str
) {
    tlv_builder(buf, 0x142, &|w| {
            w.put_u16(0);
            w.put_bytes_with_flags(apk_id.as_bytes(), PacketFlag::I16Len);
        },
    )
}

pub fn t144(
    buf: &mut BytesMut,
    tgtgt_key: &[u8],
    guid: &[u8],
    android_id: &str,
    brand: &str,
    model: &str,
    code: &str,
    os_ver: &str,
    os_type: &str,
    sim_info: &str,
    apn: &str,
) {
    tlv_builder(buf, 0x144, &|w| {
            let mut buf = BytesMut::new();
            buf.put_u16(5);
            t109(&mut buf, android_id);
            t52d(&mut buf, brand, code, os_ver, android_id);
            t124(&mut buf, os_type, os_ver, sim_info, apn);
            t128(&mut buf, false, true, false, 0x01000000, model, guid, brand);
            t16e(&mut buf, model);
            let encrypt = qqtea_encrypt(buf.as_ref(), &tgtgt_key);
            w.put_slice(&encrypt);
        },
    )
}

pub fn t145(buf: &mut BytesMut, guid: &[u8]) {
    tlv_builder(buf, 0x145, &|w| {
            w.put_slice(guid);
        },
    )
}

pub fn t147(buf: &mut BytesMut, apk_ver: &str, apk_signature_md5: &[u8]) {
    tlv_builder(buf, 0x147, &|w| {
            w.put_u32(16);
            w.put_bytes_with_flags(apk_ver.as_bytes(), PacketFlag::I16Len);
            w.put_bytes_with_flags(apk_signature_md5, PacketFlag::I16Len);
        },
    )
}

pub fn t154(buf: &mut BytesMut, seq: u32) {
    tlv_builder(buf, 0x154, &|w| {
            w.put_u32(seq);
        },
    )
}

pub fn t16a(buf: &mut BytesMut, no_pic_sig: &[u8]) {
    tlv_builder(buf, 0x16a, &|w| {
            w.put_slice(no_pic_sig);
        },
    )
}

pub fn t16e(buf: &mut BytesMut, model: &str) {
    tlv_builder(buf, 0x16e, &|w| {
            w.put_slice(model.as_bytes());
        },
    )
}

pub fn t177(buf: &mut BytesMut, build_time: u32, sdk_version: &str) {
    tlv_builder(buf, 0x177, &|w| {
            w.put_u8(0x01);
            w.put_u32(build_time);
            w.put_bytes_with_flags(sdk_version.as_bytes(), PacketFlag::I16Len);
        },
    )
}

pub fn t187(buf: &mut BytesMut, mac_address: &str) {
    tlv_builder(buf, 0x187, &|w| {
            w.put_slice(md5::compute(mac_address.as_bytes()).as_ref())
        },
    )
}

pub fn t188(buf: &mut BytesMut, android_id: &str) {
    tlv_builder(buf, 0x188, &|w| {
            w.put_slice(md5::compute(android_id.as_bytes()).as_ref())
        },
    )
}

pub fn t511(buf: &mut BytesMut, domains: &Vec<String>) {
    tlv_builder(buf, 0x511, &|w| {
        let mut arr2 = Vec::new();
        for d in domains {
            if !d.is_empty() {
                arr2.push(d)
            }
        }
        w.put_u16(arr2.len() as u16);
        for d in arr2 {
            let index_of = match d.find('(') {
                None => -1,
                Some(i) => i as isize,
            };
            let index_of2 = match d.find(')') {
                None => -1,
                Some(i) => i as isize,
            };
            if index_of != 0 || index_of2 <= 0 {
                w.put_u8(0x01);
                w.put_bytes_with_flags(d.as_bytes(), PacketFlag::I16Len)
            } else {
                let mut b: u8;
                let z: bool;
                if let Ok(i) = d[(index_of + 1) as usize..index_of2 as usize].parse::<i32>() {
                    let z2 = (1048576 & i) > 0;
                    z = (i & 134217728) > 0;
                    if z2 {
                        b = 1
                    } else {
                        b = 0
                    }
                    if z {
                        b |= 2
                    }
                    w.put_u8(b);
                    w.put_bytes_with_flags(d[(index_of2 + 1) as usize..].as_bytes(), PacketFlag::I16Len);
                }
            }
        }
    })
}

pub fn t516(buf: &mut BytesMut) {
    tlv_builder(buf, 0x516, &|w| {
            w.put_u32(0);
        },
    )
}

pub fn t521(buf: &mut BytesMut) {
    tlv_builder(buf, 0x521, &|w| {
            w.put_u32(0);
            w.put_u16(0);
        },
    )
}

pub fn t525(buf: &mut BytesMut, uin: u32, sub_app_id: u32) {
    tlv_builder(buf, 0x525, &|w| {
            w.put_u16(1);
            t536(w, uin as u32, sub_app_id);
        },
    )
}

pub fn t52d(buf: &mut BytesMut, brand: &str, code: &str, os_ver: &str, android_id: &str) {
    tlv_builder(buf, 0x52d, &|w| {
            let report = crate::pb::wtlogin::DeviceReport {
                bootloader: "unknown".to_string(),
                version: "Linux version 5.10".to_string(),
                codename: "REL".to_string(),
                incremental: "20.8.13".to_string(),
                fingerprint: format!("{}/{}/{}:{}//:user/release-keys", brand, code, code, os_ver).to_string(),
                boot_id: "".to_string(),
                android_id: android_id.to_string(),
                baseband: "".to_string(),
                inner_ver: "".to_string()
            };
            let report = report.encode_to_vec();
            w.put_slice(report.as_slice());
        },
    )
}

pub fn t533_data(buf: &mut BytesMut, data: &[u8]) {
    tlv_builder(buf, 0x533, &|w| {
            w.put_slice(data);
        },
    )
}

pub fn t536(buf: &mut BytesMut, uin: u32, sub_app_id: u32) {
    tlv_builder(buf, 0x536, &|w| {
            w.put_u8(1);
            w.put_u8(1);
            w.put_u64(uin as u64);
            w.put_u8(4);
            w.put_u32(0);
            let current_time = UNIX_EPOCH.elapsed().unwrap().as_secs();
            w.put_u32(current_time as u32);
            w.put_u32(sub_app_id);
        },
    )
}

pub fn t544(buf: &mut BytesMut, data: &[u8]) {
    tlv_builder(buf, 0x544, &|w| {
            w.put_slice(data);
        },
    )
}

pub fn t545(buf: &mut BytesMut, qimei: &str) {
    tlv_builder(buf, 0x545, &|w| {
            w.put_slice(qimei.as_bytes());
        },
    )
}

pub fn t553(buf: &mut BytesMut, data: &[u8]) {
    tlv_builder(buf, 0x553, &|w| {
            w.put_slice(data);
        },
    )
}