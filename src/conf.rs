use crate::provider::{EcbConf, IexConf};
use color_eyre::Report;
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

#[derive(Deserialize)]
pub struct Migration {
    pub version: i16,
    pub up: String,
    pub down: String,
}

impl Conf {
    pub fn new() -> Result<Conf, Report> {
        let default_conf = include_bytes!("../pfd.conf");
        let default_conf = String::from_utf8_lossy(default_conf);

        let custom_conf_path = env::var("DATA_DIR").unwrap();
        let custom_conf_path = Path::new(&custom_conf_path).join("pfd.conf");

        let conf: Conf = Figment::new()
            .merge(Toml::string(&default_conf))
            .merge(Toml::file(custom_conf_path))
            .extract()?;

        Ok(conf)
    }
}
