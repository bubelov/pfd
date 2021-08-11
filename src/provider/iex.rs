use crate::{model::ExchangeRate, provider::Provider, repository::exchange_rate};
use anyhow::Result;
use rusqlite::Connection;
use serde::Deserialize;
use std::sync::Mutex;

pub struct Iex {
    conf: IexConf,
    conn: Mutex<Connection>,
}

#[derive(Deserialize)]
pub struct IexConf {
    pub crypto: bool,
    pub crypto_schedule: String,
    pub token: String,
}

#[derive(Deserialize)]
struct IexCryptoQuote {
    #[serde(rename = "latestPrice")]
    latest_price: String,
}

impl Iex {
    pub fn new(conf: IexConf, conn: Connection) -> Iex {
        Iex {
            conf: conf,
            conn: Mutex::new(conn),
        }
    }
}

#[rocket::async_trait]
impl Provider for Iex {
    fn name(&self) -> String {
        "iex".into()
    }

    fn fiat_sync_enabled(&self) -> bool {
        false
    }

    fn fiat_sync_schedule(&self) -> String {
        "".into()
    }

    async fn sync_fiat(&self) -> Result<()> {
        Ok(())
    }

    fn crypto_sync_enabled(&self) -> bool {
        self.conf.crypto
    }

    fn crypto_sync_schedule(&self) -> String {
        self.conf.crypto_schedule.clone()
    }

    async fn sync_crypto(&self) -> Result<()> {
        let url = format!(
            "https://cloud.iexapis.com/stable/crypto/BTCEUR/quote?token={}",
            self.conf.token
        );
        let quote = reqwest::get(url).await?.json::<IexCryptoQuote>().await?;
        let rate = ExchangeRate {
            quote: "BTC".into(),
            base: "EUR".into(),
            rate: quote.latest_price.parse::<f64>()?,
        };
        exchange_rate::insert_or_replace(&rate, &mut self.conn.lock().unwrap())?;
        Ok(())
    }
}
