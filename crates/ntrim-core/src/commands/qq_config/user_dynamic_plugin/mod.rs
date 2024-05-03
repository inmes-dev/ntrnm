use log::info;
use ntrim_macros::command;

struct UserDynamicPluginService;

#[command("trpc.qq_config.user_dynamic_plugin.UserDynamicPluginService.GetList", "getDynamicPluginList", Protobuf, Service)]
impl UserDynamicPluginService {
    async fn generate(bot: &Arc<Bot>) -> Option<Vec<u8>> {
        Some(hex::decode("0801100518dae1eaa606").unwrap())
    }

    async fn parse(bot: &Arc<Bot>, data: Vec<u8>) -> Option<()> {
        Some(())
    }
}