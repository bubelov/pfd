use crate::{
    db::Db,
    model::{Error, ExchangeRate},
    repository::exchange_rates,
};
use rocket::{get, serde::json::Json};

#[get("/exchange_rates?<quote>&<base>")]
pub async fn get(base: &str, quote: &str, db: Db) -> Result<Json<ExchangeRate>, Error> {
    let quote_owned = quote.to_string();
    let base_owned = base.to_string();

    let rate = db
        .run(move |conn| exchange_rates::select_by_quote_and_base(conn, &quote_owned, &base_owned))
        .await;

    if let Err(e) = rate {
        eprintln!("{}", e);
        return Err(Error::new(500, "Internal server error"));
    }

    if let Some(v) = rate.unwrap() {
        return Ok(Json(v));
    }

    let quote_owned = quote.to_string();
    let base_owned = base.to_string();

    let rate = db
        .run(move |conn| exchange_rates::select_by_quote_and_base(conn, &base_owned, &quote_owned))
        .await;

    if let Err(e) = rate {
        eprintln!("{}", e);
        return Err(Error::new(500, "Internal server error"));
    }

    if let Some(v) = rate.unwrap() {
        return Ok(Json(ExchangeRate {
            quote: quote.to_string(),
            base: base.to_string(),
            rate: 1.0 / v.rate,
        }));
    }

    Err(Error::new(404, "Not found"))
}
