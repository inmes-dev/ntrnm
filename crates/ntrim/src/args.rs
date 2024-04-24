use clap::{Args as ClapArgs, Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    /// 配置文件路径(toml)
    #[arg(short, long)]
    pub config_path: Option<String>,
    /// 日志等级
    #[arg(short, long, default_value = "info")]
    pub log_level: String,
    /// 登录模式
    #[clap(subcommand)]
    pub login_mode: LoginMode,
}

#[derive(Debug, Subcommand)]
pub enum LoginMode {
    /// 账号密码登录
    #[clap(name = "password")]
    Password {
        /// QQ账户
        #[clap(short, long)]
        qq: String,
        /// QQ密码
        #[clap(short, long)]
        password: String,
    },
    /// 缓存会话登录(推荐)
    #[clap(name = "session")]
    Session {
        /// session文件路径(json)
        #[clap(short, long)]
        session_path: String,
    }
}
