use std::fs;
use serde::Deserialize;
use toml;

use super::error::Error;

#[derive(Deserialize, Debug)]
struct Config {
    esa: EsaInfo
}

#[derive(Deserialize, Debug)]
pub struct EsaInfo {
    pub api_key: String,
    pub team: String,
    pub user: String,
}

impl EsaInfo {
    pub fn from_file() -> Result<EsaInfo, Error> {
        let file = fs::read_to_string(".esa_cli").unwrap();
        let config: Result<Config, toml::de::Error> = toml::from_str(&file);
        return match config {
            Ok(config) => Ok(config.esa),
            Err(error) => Err(Error::EsaInfo(error)),
        };
    }
}
