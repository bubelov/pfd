use crate::provider::{EcbConf, IexConf};
use anyhow::Result;
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
    pub fn new() -> Result<Conf> {
        let default_conf = include_bytes!("../pfd.conf");
        let default_conf = String::from_utf8_lossy(default_conf);
        let conf = Figment::new().merge(Toml::string(&default_conf));

        let conf = match env::var("DATA_DIR") {
            Ok(data_dir) => {
                let custom_conf_path = Path::new(&data_dir).join("pfd.conf");
                conf.merge(Toml::file(custom_conf_path))
            }
            Err(_) => conf,
        };

        let conf: Conf = conf.extract()?;
        Ok(conf)
    }
}
