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

    if rate.is_err() {
        return Err(Error {
            code: 500,
            message: "Internal server error".to_string(),
        });
    }

    let rate = rate.unwrap();

    if rate.is_some() {
        return Ok(Json(rate.unwrap()));
    }

    let base_owned = base.to_string();
    let quote_owned = quote.to_string();

    let rate = db
        .run(move |conn| exchange_rates::select_by_base_and_quote(conn, &quote_owned, &base_owned))
        .await;

    if rate.is_err() {
        return Err(Error {
            code: 500,
            message: "Internal server error".to_string(),
        });
    }

    let rate = rate.unwrap();

    if rate.is_some() {
        return Ok(Json(ExchangeRate {
            base: base.to_string(),
            quote: quote.to_string(),
            rate: 1.0 / rate.unwrap().rate,
        }));
    }

    Err(Error {
        code: 404,
        message: "Not found".to_string(),
    })
}
