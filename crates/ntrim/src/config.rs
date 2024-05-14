use std::fs::{read_to_string, write};
use std::future::Future;
use std::path::PathBuf;
use serde_derive::{Deserialize, Serialize};
use toml::Value;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Config {
    pub qsign: QSign,
    pub developer: Developer,
    pub sql: Sql,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct QSign {
    pub server: String
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Developer {
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Sql {
    pub enable: bool,
    pub address: String,
}

pub fn parse_local_config(path: PathBuf) -> Option<Config> {
    info!("Local config file: {}", path.to_str().unwrap());
    if let Ok(contents) = read_to_string(path) {
        let config: Config = toml::from_str(&contents).unwrap();
        return Some(config);
    } else {
        error!("Failed to read config file");
    }
    None
}

/*pub fn save_config_file(path: PathBuf, config: Config) {
    let toml = toml::to_string(&config).unwrap();
    if let Err(e) = write(path, toml) {
        error!("Failed to write config file: {}", e);
    }
}*/