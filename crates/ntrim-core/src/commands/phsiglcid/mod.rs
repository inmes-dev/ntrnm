use log::info;
use ntrim_macros::command;

struct PhSigCheckBuilder;

#[command("PhSigLcId.Check", "check_phsig_lcid", Protobuf, wt_login_st)]
impl PhSigCheckBuilder {
    async fn generate(bot: &Arc<Bot>) -> Option<Vec<u8>> {
        Some(Vec::new())
    }

    async fn parse(bot: &Arc<Bot>, data: Vec<u8>) -> Option<()> {
        Some(())
    }
}