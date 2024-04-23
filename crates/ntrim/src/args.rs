use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    /// 配置文件路径(toml)
    #[arg(short, long)]
    pub config_path: Option<String>,
    /// 日志等级
    #[arg(short, long, default_value = "info")]
    pub log_level: String,
}