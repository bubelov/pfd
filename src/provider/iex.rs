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
use std::env;
use std::path::Path;
use std::str::FromStr;
use tokio::time::sleep;
use tracing::warn;

pub struct Iex {
    conf: IexConf,
    conn: Connection,
}

#[derive(Debug, Deserialize)]
struct IexConf {
    crypto: bool,
    crypto_schedule: String,
    token: String,
}

#[derive(Debug, Deserialize)]
struct IexCryptoQuote {
    #[serde(rename = "latestPrice")]
    latest_price: String,
}

impl Iex {
    pub fn new(conn: Connection) -> Result<Iex, Report> {
        let conf_path = env::var("DATA_DIR").unwrap();
        let conf_path = Path::new(&conf_path).join("pfd.conf");

        let conf: IexConf = Figment::new()
            .merge(Toml::file("pfd.conf"))
            .merge(Toml::file(conf_path))
            .extract_inner("providers.iex")?;

        Ok(Iex {
            conf: conf,
            conn: conn,
        })
    }

    pub async fn schedule(&mut self) {
        warn!(provider = "ecb", "Scheduling sync...");
        let schedule = Schedule::from_str(&self.conf.crypto_schedule).unwrap();

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
        let url = format!(
            "https://cloud.iexapis.com/stable/crypto/BTCEUR/quote?token={}",
            self.conf.token
        );
        let quote = reqwest::get(url).await?.json::<IexCryptoQuote>().await?;
        let rate = ExchangeRate {
            quote: "BTC".to_string(),
            base: "EUR".to_string(),
            rate: quote.latest_price.parse::<f64>()?,
        };
        exchange_rates::insert_or_replace(&mut self.conn, &rate)?;
        Ok(())
    }
}
