use crate::{model::ExchangeRate, repository::exchange_rates};
use chrono::Utc;
use cron::Schedule;
use rocket::figment::Figment;
use rocket::serde::Deserialize;
use rusqlite::Connection;
use std::error::Error;
use std::str::FromStr;
use tokio::time::sleep;

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
    pub fn new(conf: &Figment, conn: Connection) -> IexCryptoProvider {
        let conf: IexCryptoProviderConf = conf.extract_inner("providers.crypto.iex").unwrap();

        IexCryptoProvider {
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
        exchange_rates::insert_or_replace(&mut self.conn, &rate);
        Ok(())
    }
}
