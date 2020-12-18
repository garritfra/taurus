use serde::Deserialize;
use std::{fs, path::Path};

#[derive(Deserialize)]
pub struct Config {
    pub port: Option<u16>,

    pub certificate_file: Option<String>,
    pub certificate_password: String,

    pub static_root: Option<String>,
}

impl Config {
    pub fn load<P: AsRef<Path>>(config_path: P) -> anyhow::Result<Self> {
        let buf = fs::read_to_string(config_path)?;
        toml::from_str(&buf).map_err(|e| e.into())
    }
}
