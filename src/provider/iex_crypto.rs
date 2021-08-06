use crate::{model::ExchangeRate, repository::exchange_rates};
use chrono::Utc;
use color_eyre::Report;
use cron::Schedule;
use rocket::figment::Figment;
use rocket::serde::Deserialize;
use rusqlite::Connection;
use std::str::FromStr;
use tokio::time::sleep;
use tracing::warn;

pub struct IexCryptoProvider {
    pub conf: IexCryptoProviderConf,
    conn: Connection,
}

#[derive(Debug, Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct IexCryptoProviderConf {
    pub enabled: bool,
    schedule: String,
    token: String,
}

#[derive(Debug, Deserialize)]
#[serde(crate = "rocket::serde")]
struct IexCryptoQuote {
    #[serde(rename = "latestPrice")]
    latest_price: String,
}

impl IexCryptoProvider {
    pub fn new(conf: &Figment, conn: Connection) -> Result<IexCryptoProvider, Report> {
        let conf: IexCryptoProviderConf = conf.extract_inner("providers.crypto.iex")?;

        Ok(IexCryptoProvider {
            conf: conf,
            conn: conn,
        })
    }

    pub async fn schedule(&mut self) {
        warn!(provider = "ecb", "Scheduling sync...");
        let schedule = Schedule::from_str(&self.conf.schedule).unwrap();

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
