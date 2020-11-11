use serde::Deserialize;
use std::fs;
use std::io::prelude::*;

#[derive(Deserialize)]
pub struct Config {
    pub port: Option<u16>,

    pub certificate_file: Option<String>,
    pub certificate_password: String,

    pub static_root: Option<String>,
}

impl Config {
    pub fn load(config_path: &str) -> anyhow::Result<Self> {
        let mut file = fs::File::open(config_path)?;
        let mut contents = String::new();

        file.read_to_string(&mut contents)?;

        Ok(toml::from_str(&contents)?)
    }
}
