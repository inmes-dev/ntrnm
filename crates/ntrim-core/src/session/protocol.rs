use once_cell::sync::Lazy;

#[derive(Debug, Clone)]
pub struct Protocol {
    pub app_id: u32,
    pub detail: String,
    pub nt_build_version: String,
}

pub static QQ_9_0_20: Lazy<Protocol> = Lazy::new(|| {
    Protocol {
        app_id: 0x20051ed6,
        detail: "||A9.0.20.38faf5bf".to_string(),
        nt_build_version: "15515".to_string(),
    }
});