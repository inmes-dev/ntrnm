#[derive(Debug, Clone)]
pub struct Protocol {
    pub sub_app_id: u32,
    pub detail: String,
    pub nt_build_version: String,
    pub apk_id: String,
    pub apk_ver: String,
    pub apk_sign: [u8; 16],
    pub misc_bitmap: u32,
    pub sub_sig_map: u32,
    pub sso_version: u32,
    pub main_sig_map: u32,
    pub locale_id: u32,
    pub build_time: u32,
    pub sdk_version: String,
}

pub mod protocol {
    use std::sync::OnceLock;
    use crate::session::protocol::Protocol;

    pub fn qq_9_0_20() -> &'static Protocol {
        static QQ_9_0_20: OnceLock<Protocol> = OnceLock::new();
        QQ_9_0_20.get_or_init(|| {
            Protocol {
                sub_app_id: 0x20051ed6,
                detail: "||A9.0.20.38faf5bf".to_string(),
                nt_build_version: "15515".to_string(),
                apk_id: "com.tencent.mobileqq".to_string(),
                apk_ver: "9.0.20".to_string(),
                apk_sign: [
                    0xa6, 0xb7, 0x45, 0xbf,
                    0x24, 0xa2, 0xc2, 0x77,
                    0x52, 0x77, 0x16, 0xf6,
                    0xf3, 0x6e, 0xb6, 0x8d
                ],
                misc_bitmap: 184024956,
                sub_sig_map: 0x10400,
                sso_version: 21,
                main_sig_map: 34869472,
                locale_id: 2052,
                build_time: 0x65800651,
                sdk_version: "6.0.0.2558".to_string(),
            }
        })
    }
}
