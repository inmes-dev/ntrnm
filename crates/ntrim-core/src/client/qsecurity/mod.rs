use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use once_cell::sync::Lazy;

#[derive(Debug, Clone)]
pub struct QSecurityResult {
    pub(crate) sign: Box<Vec<u8>>,
    pub(crate) extra: Box<Vec<u8>>,
    pub(crate) token: Box<Vec<u8>>,
}

pub static EMPTY_QSECURITY_RESULT: Lazy<QSecurityResult> = Lazy::new(||{
    QSecurityResult {
        sign: Box::new(Vec::new()),
        extra: Box::new(Vec::new()),
        token: Box::new(Vec::new()),
    }
});

impl QSecurityResult {
    pub(crate) fn new(sign: Box<Vec<u8>>, extra: Box<Vec<u8>>, token: Box<Vec<u8>>) -> Self {
        Self { sign, extra, token }
    }

    #[inline]
    pub(crate) fn new_empty() -> Self {
        EMPTY_QSECURITY_RESULT.clone()
    }
}

static WHITELIST_COMMANDS: [&str; 102] = [
    "trpc.o3.ecdh_access.EcdhAccess.SsoSecureA2Establish",
    "trpc.o3.ecdh_access.EcdhAccess.SsoSecureA2Access",
    "OidbSvcTrpcTcp.0xf88_1",
    "OidbSvcTrpcTcp.0x1105_1",
    "trpc.o3.report.Report.SsoReport",
    "wtlogin.trans_emp",
    "OidbSvcTrpcTcp.0xf89_1",
    "wtlogin_device.login",
    "trpc.commercial.dataworks.UserActionReport_sso.SsoReport",
    "OidbSvcTrpcTcp.0xf67_5",
    "OidbSvcTrpcTcp.0xfa5_1",
    "OidbSvcTrpcTcp.0x55f_0",
    "wtlogin.device_lock",
    "qidianservice.207",
    "wtlogin_device.tran_sim_emp",
    "OidbSvc.0x758_0",
    "SQQzoneSvc.addReply",
    "trpc.o3.ecdh_access.EcdhAccess.SsoEstablishShareKey",
    "OidbSvc.0x89a_0",
    "trpc.passwd.manager.PasswdManager.SetPasswd",
    "QQConnectLogin.pre_auth",
    "trpc.qlive.word_svr.WordSvr.NewPublicChat",
    "OidbSvc.0x8a1_0",
    "SQQzoneSvc.publishmood",
    "OidbSvcTrpcTcp.0x101e_2",
    "OidbSvcTrpcTcp.0x101e_1",
    "OidbSvcTrpcTcp.0xf67_1",
    "OidbSvcTrpcTcp.0xf6e_1",
    "OidbSvc.0x8a1_7",
    "OidbSvc.0x758_1",
    "OidbSvc.0x4ff_9",
    "OidbSvc.0x88d_0",
    "FeedCloudSvr.trpc.feedcloud.commwriter.ComWriter.DoLike",
    "SQQzoneSvc.addComment",
    "MessageSvc.PbSendMsg",
    "FeedCloudSvr.trpc.feedcloud.commwriter.ComWriter.DoComment",
    "SQQzoneSvc.shuoshuo",
    "SQQzoneSvc.like",
    "trpc.o3.ecdh_access.EcdhAccess.SsoSecureAccess",
    "OidbSvc.0x758",
    "QChannelSvr.trpc.qchannel.commwriter.ComWriter.DoComment",
    "QChannelSvr.trpc.qchannel.commwriter.ComWriter.DoReply",
    "wtlogin.qrlogin",
    "OidbSvcTrpcTcp.0xf57_1",
    "OidbSvc.oidb_0x758",
    "OidbSvcTrpcTcp.0xf57_9",
    "wtlogin.exchange_emp",
    "OidbSvc.0x56c_6",
    "QChannelSvr.trpc.qchannel.commwriter.ComWriter.PublishFeed",
    "OidbSvcTrpcTcp.0xf55_1",
    "OidbSvcTrpcTcp.0x6d9_4",
    "trpc.qlive.relationchain_svr.RelationchainSvr.Follow",
    "ProfileService.GroupMngReq",
    "ProfileService.getGroupInfoReq",
    "ConnAuthSvr.get_app_info_emp",
    "OidbSvcTrpcTcp.0x1100_1",
    "FeedCloudSvr.trpc.videocircle.circleprofile.CircleProfile.SetProfile",
    "qidianservice.135",
    "trpc.group_pro.msgproxy.sendmsg",
    "FeedCloudSvr.trpc.feedcloud.commwriter.ComWriter.DoPush",
    "FeedCloudSvr.trpc.feedcloud.commwriter.ComWriter.DoReply",
    "FeedCloudSvr.trpc.feedcloud.commwriter.ComWriter.DoBarrage",
    "QQConnectLogin.get_promote_page",
    "friendlist.addFriend",
    "SQQzoneSvc.forward",
    "OidbSvc.0x4ff_9_IMCore",
    "OidbSvc.0x6d9_4",
    "trpc.springfestival.redpacket.LuckyBag.SsoSubmitGrade",
    "wtlogin.login",
    "OidbSvc.0x89b_1",
    "trpc.qqhb.qqhb_proxy.Handler.sso_handle",
    "qidianservice.290",
    "wtlogin.register",
    "OidbSvc.0x8ba",
    "ConnAuthSvr.sdk_auth_api_emp",
    "OidbSvc.0x9fa",
    "qidianservice.269",
    "OidbSvcTrpcTcp.0xf65_1",
    "FeedCloudSvr.trpc.feedcloud.commwriter.ComWriter.PublishFeed",
    "OidbSvcTrpcTcp.0xf65_10",
    "ConnAuthSvr.fast_qq_login",
    "trpc.login.ecdh.EcdhService.SsoQRLoginGenQr",
    "friendlist.AddFriendReq",
    "MsgProxy.SendMsg",
    "trpc.login.ecdh.EcdhService.SsoNTLoginPasswordLoginUnusualDevice",
    "trpc.passwd.manager.PasswdManager.VerifyPasswd",
    "trpc.login.ecdh.EcdhService.SsoQRLoginScanQr",
    "QQConnectLogin.get_promote_page_emp",
    "FeedCloudSvr.trpc.feedcloud.commwriter.ComWriter.DoFollow",
    "QQConnectLogin.submit_promote_page",
    "friendlist.ModifyGroupInfoReq",
    "OidbSvcTrpcTcp.0xf57_106",
    "QQConnectLogin.submit_promote_page_emp",
    "OidbSvcTrpcTcp.0x1107_1",
    "QQConnectLogin.pre_auth_emp",
    "ConnAuthSvr.get_app_info",
    "ConnAuthSvr.sdk_auth_api",
    "wtlogin.name2uin",
    "QQConnectLogin.auth",
    "ConnAuthSvr.get_auth_api_list_emp",
    "QQConnectLogin.auth_emp",
    "ConnAuthSvr.get_auth_api_list"
];

pub trait QSecurity: Send + Sync {
    fn is_whitelist_command<'a>(&'a self, cmd: &'a str) -> Pin<Box<dyn Future<Output = bool> + Send + 'a>> {
        Box::pin(async move {
            WHITELIST_COMMANDS.contains(&cmd)
        })
    }

    fn energy<'a>(&'a self, data: String, salt: Box<[u8]>) -> Pin<Box<dyn Future<Output = Vec<u8>> + Send + 'a>>;

    fn sign<'a>(&'a self, uin: String, cmd: String, buffer: Arc<Vec<u8>>, seq: u32) -> Pin<Box<dyn Future<Output = QSecurityResult> + Send + 'a>>;
}