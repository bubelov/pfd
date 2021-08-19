use crate::{model::ExchangeRate, provider::Provider, repository::ExchangeRateRepository};
use anyhow::Result;
use serde::Deserialize;

pub struct Iex {
    conf: IexConf,
    repo: ExchangeRateRepository,
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
    pub fn new(conf: IexConf, repo: ExchangeRateRepository) -> Iex {
        Iex {
            conf: conf,
            repo: repo,
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
        self.repo.insert_or_replace(&rate)?;
        Ok(())
    }
}
