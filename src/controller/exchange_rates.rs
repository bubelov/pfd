use crate::{
    db::Db,
    model::{Error, ExchangeRate},
    repository::exchange_rates,
};
use rocket::{get, serde::json::Json};

#[get("/exchange_rates?<base>&<quote>")]
pub async fn get(base: &str, quote: &str, db: Db) -> Result<Json<ExchangeRate>, Error> {
    let base_owned = base.to_string();
    let quote_owned = quote.to_string();

    let rate = db
        .run(move |conn| exchange_rates::select_by_base_and_quote(conn, &base_owned, &quote_owned))
        .await;

    if let Err(e) = rate {
        eprintln!("{}", e);
        return Err(Error::new(500, "Internal server error"));
    }

    if let Some(v) = rate.unwrap() {
        return Ok(Json(v));
    }

    let base_owned = base.to_string();
    let quote_owned = quote.to_string();

    let rate = db
        .run(move |conn| exchange_rates::select_by_base_and_quote(conn, &quote_owned, &base_owned))
        .await;

    if let Err(e) = rate {
        eprintln!("{}", e);
        return Err(Error::new(500, "Internal server error"));
    }

    if let Some(v) = rate.unwrap() {
        return Ok(Json(ExchangeRate {
            base: base.to_string(),
            quote: quote.to_string(),
            rate: 1.0 / v.rate,
        }));
    }

    Err(Error::new(404, "Not found"))
}
