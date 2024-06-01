use std::sync::Arc;
use log::{error, info};
use tokio::sync::mpsc::Sender;
use tokio::sync::oneshot;
use crate::client::codec;
use crate::client::packet::{FromServiceMsg, ToServiceMsg};
use crate::client::packet::packet::CommandType::{ExchangeSig, ExchangeSt, Register, Service};
use crate::client::packet::packet::UniPacket;
use crate::client::trpc::TrpcClient;
use crate::commands;
use crate::session::ticket::{SigType, TicketManager};

impl TrpcClient {
    pub async fn send_uni_packet(self: &Arc<TrpcClient>, uni_packet: UniPacket) -> (u32, Option<oneshot::Receiver<FromServiceMsg>>) {
        let session = self.session.clone();
        let session = session.read().await;
        let seq = session.next_seq();
        return (seq, self.send_uni_packet_with_seq(uni_packet, seq).await);
    }

    pub async fn send_uni_packet_with_seq(self: &Arc<TrpcClient>, uni_packet: UniPacket, seq: u32) -> Option<oneshot::Receiver<FromServiceMsg>> {
        if !self.is_connected().await || self.is_lost().await {
            return None;
        }

        let (tx, rx) = oneshot::channel();
        let session = self.session.clone();
        let session = session.read().await;

        let cmd = uni_packet.command.clone();

        if cmd != "trpc.qq_new_tech.status_svc.StatusService.SsoHeartBeat" {
            unsafe {
                codec::LAST_PACKET_TIME = chrono::Utc::now().timestamp();
            }
        }

        let sec_info = if self.qsec.is_whitelist_command(cmd.as_str()).await {
            Some(self.qsec.sign(
                session.uin.to_string(),
                uni_packet.command.clone(),
                uni_packet.wup_buffer.clone(),
                seq
            ).await)
        } else {
            None
        };

        let mut msg = ToServiceMsg::new(uni_packet, seq);
        if let Some(sec_info) = sec_info {
            if sec_info.sign.is_empty() {
                error!("Failed to sign packet, seq: {}, cmd: {}", seq, cmd);
                return None;
            } else {
                msg.sec_info = Option::from(sec_info);
            }
        }

        if *codec::enable_print_codec_logs() {
            info!("Send packet, cmd: {}, seq: {}", cmd, seq);
        }

        match msg.uni_packet.command_type {
            Register => {
                let d2 = session.ticket(SigType::D2).unwrap();
                let d2 = d2.sig.clone().unwrap();
                msg.first_token = Some(Box::new(d2));
                let tgt = session.ticket(SigType::A2).unwrap();
                let tgt = tgt.sig.clone().unwrap();
                msg.second_token = Some(Box::new(tgt));
            }
            Service => {
                // nothing
            }
            ExchangeSt => {
                // nothing
            }
            ExchangeSig => {
                // nothing
            }
            _ => {
                error!("Invalid command type: {:?}", msg.uni_packet.command_type);
            }
        }
        self.dispatcher.register_oneshot(seq, tx).await;
        if let Err(e) = self.sender.send(msg).await {
            error!("Failed to send packet account: {:?} ,err: {}", session.uin, e);
            return None;
        }
        return Some(rx);
    }

    pub async fn unregister_oneshot(self: &Arc<TrpcClient>, seq: u32) {
        self.dispatcher.unregister_oneshot(seq).await;
    }

    pub async fn register_persistent(self: &Arc<TrpcClient>, cmd: String, sender: Sender<FromServiceMsg>) {
        self.dispatcher.register_persistent(cmd, sender).await;
    }

    pub async fn register_multiple_persistent(self: &Arc<TrpcClient>, cmd: Vec<String>, sender: Sender<FromServiceMsg>) {
        self.dispatcher.register_multiple_persistent(cmd, sender).await;
    }
}