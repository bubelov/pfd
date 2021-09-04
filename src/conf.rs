use crate::provider::{EcbConf, IexConf};
use anyhow::{ensure, Context, Result};
use figment::{
    providers::{Format, Toml},
    Figment,
};
use serde::Deserialize;
use std::{
    env,
    {include_bytes, path::Path},
};

#[derive(Deserialize)]
pub struct Conf {
    pub db_url: String,
    pub providers: ProvidersConf,
    pub migrations: Vec<Migration>,
}

#[derive(Deserialize)]
pub struct ProvidersConf {
    pub ecb: EcbConf,
    pub iex: IexConf,
}

#[derive(Clone, Deserialize)]
pub struct Migration {
    pub version: i16,
    pub up: String,
    pub down: String,
}

impl Conf {
    pub fn new() -> Result<Self> {
        let db_url = env::var("DATA_DIR").with_context(|| "DATA_DIR isn't set")?;
        let db_url = Path::new(&db_url).join("pfd.db");

        let conf = include_bytes!("../pfd.conf");
        let conf = String::from_utf8_lossy(conf);

        let conf = Figment::new()
            .merge(Toml::string(&conf))
            .merge(("db_url", db_url));

        let conf = match env::var("DATA_DIR") {
            Ok(data_dir) => {
                let custom_conf_path = Path::new(&data_dir).join("pfd.conf");
                conf.merge(Toml::file(custom_conf_path))
            }
            Err(_) => conf,
        };

        let conf: Self = conf
            .extract()
            .with_context(|| "Failed to deserialize config")?;

        ensure!(
            !conf.providers.iex.crypto || conf.providers.iex.token.len() > 0,
            "IEX provider is enabled but token isn't set"
        );

        Ok(conf)
    }
}
