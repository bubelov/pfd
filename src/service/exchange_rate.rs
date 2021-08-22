use crate::{model::ExchangeRate, repository::ExchangeRateRepository};
use anyhow::Result;

pub struct ExchangeRateService {
    repo: ExchangeRateRepository,
}

impl ExchangeRateService {
    pub fn new(repo: &ExchangeRateRepository) -> ExchangeRateService {
        ExchangeRateService { repo: repo.clone() }
    }

    pub fn get_by_quote_and_base(&self, quote: &str, base: &str) -> Result<Option<ExchangeRate>> {
        let rate = self.repo.select_by_quote_and_base(&quote, &base);

        if let Some(v) = rate? {
            return Ok(Some(v));
        }

        let rate = self.repo.select_by_quote_and_base(&base, &quote);

        if let Some(v) = rate? {
            return Ok(Some(ExchangeRate {
                quote: quote.to_string(),
                base: base.to_string(),
                rate: 1.0 / v.rate,
            }));
        }

        let indirect_rate_1 = self
            .repo
            .select_by_quote_and_base(&quote, &"EUR".to_string())?;
        let indirect_rate_2 = self
            .repo
            .select_by_quote_and_base(&base, &"EUR".to_string())?;

        if indirect_rate_1.is_some() && indirect_rate_2.is_some() {
            return Ok(Some(ExchangeRate {
                quote: quote.into(),
                base: base.into(),
                rate: indirect_rate_1.unwrap().rate / indirect_rate_2.unwrap().rate,
            }));
        }

        Ok(None)
    }
}
