use crate::{model::ExchangeRate, repository::exchange_rates};
use chrono::Utc;
use color_eyre::Report;
use cron::Schedule;
use figment::{
    providers::{Format, Toml},
    Figment,
};
use rusqlite::Connection;
use serde::Deserialize;
use std::io::{copy, Cursor};
use std::str::FromStr;
use tokio::time::sleep;
use tracing::warn;
use zip::ZipArchive;

pub struct Ecb {
    conf: EcbConf,
    conn: Connection,
}

#[derive(Debug, Deserialize)]
struct EcbConf {
    fiat: bool,
    fiat_schedule: String,
}

impl Ecb {
    pub fn new(conn: Connection) -> Result<Ecb, Report> {
        let conf: EcbConf = Figment::new()
            .merge(Toml::file("pfd.conf"))
            .extract_inner("providers.ecb")?;

        Ok(Ecb {
            conf: conf,
            conn: conn,
        })
    }

    pub async fn schedule(&mut self) {
        warn!(provider = "ecb", "Scheduling sync...");
        let schedule = Schedule::from_str(&self.conf.fiat_schedule).unwrap();

        for next_sync in schedule.upcoming(Utc) {
            warn!(provider = "ecb", %next_sync, "Got next sync date");
            let time_to_next_sync = next_sync.signed_duration_since(Utc::now());
            if time_to_next_sync.num_nanoseconds().unwrap() < 0 {
                warn!("Skipping next sync because the old one didn't finish in time");
                continue;
            }
            let time_to_next_sync = time_to_next_sync.to_std().unwrap();
            warn!(
                secs_to_next_sync = time_to_next_sync.as_secs(),
                "Going to sleep till next sync"
            );
            sleep(time_to_next_sync).await;
            warn!("Syncing...");
            self.sync().await.unwrap();
        }
    }

    pub async fn sync(&mut self) -> Result<(), Report> {
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
            exchange_rates::insert_or_replace(&mut self.conn, &rate)?;
        }

        Ok(())
    }
}
