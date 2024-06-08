use std::sync::Arc;
use bytes::{BufMut, BytesMut};
use ntrim_tools::bytes::{BytePacketBuilder, PacketFlag};
use crate::client::packet::packet::CommandType;
use crate::client::qsecurity::QSecurity;
use crate::client::trpc::TrpcClient;
use crate::commands::wtlogin::tlv::{*};
use crate::commands::wtlogin::wtlogin_request::{WtloginFactory, WtloginBuilder};
use crate::commands::wtlogin::WtloginRequest;
use crate::session::SsoSession;
use crate::session::ticket::{TicketManager};

/// 转录F受体对应的DNA
/// RNA翻译表达F受体
/// 释放信号请求
pub struct RefreshSig {
    pub aid: u32,
    pub domains: Vec<String>
}

impl WtloginFactory<RefreshSig> for WtloginBuilder<RefreshSig> {
    type Params = (u32, Vec<String>);

    fn build(
        trpc: Arc<TrpcClient>,
        params: Self::Params
    ) -> Arc<WtloginBuilder<RefreshSig>> {
        Arc::new(WtloginBuilder {
            trpc,
            command: "wtlogin.exchange_emp".to_string(),
            command_type: CommandType::ExchangeSig,
            wt_command: 0x810,
            wt_sub_command: 0xf,
            request: RefreshSig {
                aid: params.0,
                domains: params.1
            }
        })
    }
}

/// 使用示例：
///let domains = vec![
///    "office.qq.com".to_string(),
///    "qun.qq.com".to_string(),
///    "gamecenter.qq.com".to_string(),
///    "docs.qq.com".to_string(),
///    "mail.qq.com".to_string(),
///    "tim.qq.com".to_string(),
///    "ti.qq.com".to_string(),
///    "vip.qq.com".to_string(),
///    "tenpay.com".to_string(),
///    "qqweb.qq.com".to_string(),
///    "qzone.qq.com".to_string(),
///    "mma.qq.com".to_string(),
///    "game.qq.com".to_string(),
///    "openmobile.qq.com".to_string(),
///    "connect.qq.com".to_string()
///];
/// // 这个会比map + to_string + collect快那么一点点
///let rx = WtloginBuilder::build(bot.client.clone(), (16, domains))
///.send().await;
///let resp = rx.await.unwrap();
///info!("Refresh sig response: {:?}", resp);
impl WtloginRequest for RefreshSig {
    fn get_encrypt_key(&self, session: &SsoSession) -> (u8, Vec<u8>, Vec<u8>) {
        let st_session_ticket  = session.wt_session_ticket.clone();
        let st_session_key = session.wt_session_key.clone();
        (0x45, st_session_ticket, st_session_key)
    }

    async fn generate_tlv544(&self, session: &SsoSession, qsec: Arc<dyn QSecurity>) -> Vec<u8> {
        let data = "810_f".to_string();
        let sdk_version = session.protocol.sdk_version.clone();
        let mut salt = BytesMut::new();
        salt.put_u32(0);
        salt.put_bytes_with_flags(&session.guid, PacketFlag::I16Len);
        salt.put_bytes_with_flags(sdk_version.as_bytes(), PacketFlag::I16Len);
        salt.put_u16(0xf);
        salt.put_u32(0);
        let salt = salt.to_vec();
        qsec.energy(data, salt).await
    }

    async fn generate_tlv_body(
        &self,
        session: &SsoSession,
        qsec: Arc<dyn QSecurity>,
        wt_command: u16,
        seq: u32
    ) -> Vec<u8> {
        let uin = session.uin as u32;
        let protocol = &session.protocol;
        let device = &session.device;

        let mut buf = BytesMut::new();
        buf.put_u16(wt_command);

        buf.put_u16(25);
        t18(&mut buf, uin);
        t1(&mut buf, uin);
        t106_data(&mut buf, session.encrypt_a1.as_slice());
        t116(&mut buf, protocol.misc_bitmap, protocol.sub_sig_map);
        t100(&mut buf, protocol.sso_version, protocol.sub_app_id, protocol.main_sig_map, self.aid);
        t107(&mut buf);
        t108(&mut buf, &session.ksid);
        t144(
            &mut buf,
            session.tgtgt_key.as_slice(),
            &session.guid,
            &device.android_id,
            &device.brand,
            &device.device_name,
            &device.code,
            &device.os_ver,
            &device.os_type,
            &device.apn_name,
            &device.apn
        );
        t142(&mut buf, &protocol.apk_id);
        t145(&mut buf, &session.guid);
        t16a(&mut buf, session.no_pic_sig.as_slice());
        t154(&mut buf, seq);
        t141(&mut buf, &device.apn_name, &device.apn);
        t8(&mut buf, protocol.locale_id);
        t511(&mut buf, &self.domains);
        t147(&mut buf, &protocol.apk_ver, &protocol.apk_sign);
        t177(&mut buf, protocol.build_time, &protocol.sdk_version);
        t187(&mut buf, &device.mac_address);
        t188(&mut buf, &device.android_id);
        t516(&mut buf);
        t521(&mut buf);
        t525(&mut buf, uin, protocol.sub_app_id);

        let tlv544 = self.generate_tlv544(session, qsec).await;
        t544(&mut buf, tlv544.as_slice());

        t553(&mut buf, device.fingerprint.as_slice());
        t545(&mut buf, &device.qimei);

        buf.to_vec()
    }
}