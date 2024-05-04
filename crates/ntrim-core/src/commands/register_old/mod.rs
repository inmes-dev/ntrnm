use log::info;
use prost::Message;
use ntrim_macros::command;
use pb::trpc::register::{ * };
use crate::pb;

struct RegisterBuilder;

#[command("trpc.qq_new_tech.status_svc.StatusService.Register", "registerNt", Protobuf, Register)]
impl RegisterBuilder {
    async fn generate(bot: &Arc<Bot>) -> Option<Vec<u8>> {
        let session = bot.client.session.clone();
        let mut session = session.write().await;
        info!("Generating register request for bot: {:?}", session.uid);
        let protocol = &(session.protocol);
        let device = &(session.device);

        let mut register_info = RegisterInfo::default();
        register_info.guid = hex::encode(session.guid.as_slice());
        register_info.kick_pc = 0;
        register_info.build_ver = protocol.nt_build_version.to_string();
        register_info.is_first_register_proxy_online = 0;
        register_info.locale_id = 2052;
        register_info.device_info = DeviceInfo {
            dev_name: format!("{}-{}", device.brand, device.device_name),
            dev_type: device.brand.to_string(),
            os_ver: device.os_ver.to_string(),
            brand: device.brand.to_string(),
            vendor_os_name: "V140".to_string(),
        };
        register_info.set_mut = 0;
        register_info.register_vendor_type = 0;
        register_info.reg_type = 1;
        Some(register_info.encode_to_vec())
    }

    async fn parse(bot: &Arc<Bot>, data: Vec<u8>) -> Option<SsoSyncInfoResponse> {
        // not nt reigster req
        let str = String::from_utf8(data).unwrap();
        println!("recv: {}", str);

        Some(SsoSyncInfoResponse::default())
    }
}
