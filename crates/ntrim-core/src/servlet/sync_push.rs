use std::sync::Arc;
use log::info;
use crate::bot::Bot;

pub struct SyncPushServlet(Arc<Bot>);

impl SyncPushServlet {
    pub async fn register(bot: &Arc<Bot>) {
        let servlet = Arc::new(Self(bot.clone()));
        let (tx, mut rx) = tokio::sync::mpsc::channel(1);
        bot.client.register_persistent("trpc.msg.register_proxy.RegisterProxy.SsoInfoSync".to_string(), tx).await;
        info!("Registered sync push servlet");
        tokio::spawn(async move {
            loop {
                if let Some(from) = rx.recv().await {

                }
            }
        });
    }
}


