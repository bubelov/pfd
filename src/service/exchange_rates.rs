use crate::{db::Db, model::ExchangeRate, repository::exchange_rates};
use std::error::Error;

pub async fn get(quote: &str, base: &str, db: Db) -> Result<Option<ExchangeRate>, Box<dyn Error>> {
    let quote_owned = quote.to_string();
    let base_owned = base.to_string();

    let rate = db
        .run(move |conn| exchange_rates::select_by_quote_and_base(conn, &quote_owned, &base_owned))
        .await;

    if let Some(v) = rate? {
        return Ok(Some(v));
    }

    let quote_owned = quote.to_string();
    let base_owned = base.to_string();

    let rate = db
        .run(move |conn| exchange_rates::select_by_quote_and_base(conn, &base_owned, &quote_owned))
        .await;

    if let Some(v) = rate? {
        return Ok(Some(ExchangeRate {
            quote: quote.to_string(),
            base: base.to_string(),
            rate: 1.0 / v.rate,
        }));
    }

    let quote_owned = quote.to_string();
    let base_owned = base.to_string();
    let indirect_rate_1 = db
        .run(move |conn| {
            exchange_rates::select_by_quote_and_base(conn, &quote_owned, &"EUR".to_string())
        })
        .await?;
    let indirect_rate_2 = db
        .run(move |conn| {
            exchange_rates::select_by_quote_and_base(conn, &base_owned, &"EUR".to_string())
        })
        .await?;

    if indirect_rate_1.is_some() && indirect_rate_2.is_some() {
        return Ok(Some(ExchangeRate {
            quote: quote.to_string(),
            base: base.to_string(),
            rate: indirect_rate_1.unwrap().rate / indirect_rate_2.unwrap().rate,
        }));
    }

    Ok(None)
}
