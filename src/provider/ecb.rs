use crate::{model::ExchangeRate, provider::Provider, repository::exchange_rate};
use anyhow::Result;
use rusqlite::Connection;
use serde::Deserialize;
use std::{
    io::{copy, Cursor},
    sync::Mutex,
};
use zip::ZipArchive;

pub struct Ecb {
    conf: EcbConf,
    conn: Mutex<Connection>,
}

#[derive(Deserialize)]
pub struct EcbConf {
    pub fiat: bool,
    pub fiat_schedule: String,
}

impl Ecb {
    pub fn new(conf: EcbConf, conn: Connection) -> Ecb {
        Ecb {
            conf: conf,
            conn: Mutex::new(conn),
        }
    }
}

#[rocket::async_trait]
impl Provider for Ecb {
    fn name(&self) -> String {
        "ecb".into()
    }

    fn fiat_sync_enabled(&self) -> bool {
        self.conf.fiat
    }

    fn fiat_sync_schedule(&self) -> String {
        self.conf.fiat_schedule.clone()
    }

    async fn sync_fiat(&self) -> Result<()> {
        let url = "https://www.ecb.europa.eu/stats/eurofxref/eurofxref.zip";
        let res = reqwest::get(url).await?;
        let body = Cursor::new(res.bytes().await?);
        let mut archive = ZipArchive::new(body)?;
        let mut compressed_csv = archive.by_index(0)?;
        let mut csv: Vec<u8> = vec![];
        copy(&mut compressed_csv, &mut csv)?;
        let csv = String::from_utf8(csv)?;
        let lines: Vec<&str> = csv.lines().collect();
        let headers: Vec<&str> = lines[0].strip_suffix(", ").unwrap().split(", ").collect();
        let codes = &headers[1..];
        let values: Vec<&str> = lines[1].strip_suffix(", ").unwrap().split(", ").collect();
        let rates = &values[1..];

        let rates: Vec<ExchangeRate> = codes
            .iter()
            .zip(rates.iter())
            .map(|(code, rate)| ExchangeRate {
                quote: code.to_string(),
                base: "EUR".to_string(),
                rate: 1.0 / rate.parse::<f64>().unwrap(),
            })
            .collect();

        for rate in rates {
            exchange_rate::insert_or_replace(&rate, &mut self.conn.lock().unwrap())?;
        }

        Ok(())
    }

    fn crypto_sync_enabled(&self) -> bool {
        false
    }

    fn crypto_sync_schedule(&self) -> String {
        "".into()
    }

    async fn sync_crypto(&self) -> Result<()> {
        Ok(())
    }
}
