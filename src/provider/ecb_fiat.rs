use crate::{model::ExchangeRate, repository::exchange_rates};
use chrono::Utc;
use cron::Schedule;
use rocket::figment::Figment;
use rocket::serde::Deserialize;
use rusqlite::Connection;
use std::str::FromStr;
use std::{
    error::Error,
    io::{copy, Cursor},
};
use tokio::time::sleep;
use zip::ZipArchive;

pub struct EcbFiatProvider {
    pub conf: EcbFiatProviderConf,
    conn: Connection,
}

#[derive(Debug, Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct EcbFiatProviderConf {
    pub enabled: bool,
    schedule: String,
}

impl EcbFiatProvider {
    pub fn new(conf: &Figment, conn: Connection) -> EcbFiatProvider {
        let conf: EcbFiatProviderConf = conf.extract_inner("providers.fiat.ecb").unwrap();

        EcbFiatProvider {
            conf: conf,
            conn: conn,
        }
    }

    pub async fn schedule(&mut self) {
        println!("Scheduling sync...");
        let schedule = Schedule::from_str(&self.conf.schedule).unwrap();

        for next_sync in schedule.upcoming(Utc) {
            println!("Next sync: {}", next_sync);
            let time_to_next_sync = next_sync.signed_duration_since(Utc::now());
            if time_to_next_sync.num_nanoseconds().unwrap() < 0 {
                println!("Skipping next sync because the old one didn't finish in time");
                continue;
            }
            let time_to_next_sync = time_to_next_sync.to_std().unwrap();
            println!(
                "Time to next sync in seconds {}",
                time_to_next_sync.as_secs()
            );
            sleep(time_to_next_sync).await;
            println!("Syncing...");
            self.sync().await.unwrap();
        }
    }

    pub async fn sync(&mut self) -> Result<(), Box<dyn Error>> {
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
            exchange_rates::insert_or_replace(&mut self.conn, &rate);
        }

        Ok(())
    }
}
