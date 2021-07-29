use crate::{
    db::Db,
    model::{Error, ExchangeRate},
    repository::exchange_rates,
};
use rocket::{get, serde::json::Json};

#[get("/exchange_rates?<base>&<quote>")]
pub async fn get(base: &str, quote: &str, db: Db) -> Result<Json<ExchangeRate>, Error> {
    let base = base.to_string();
    let quote = quote.to_string();

    db.run(move |c| exchange_rates::select_by_base_and_quote(c, &base, &quote))
        .await
        .map(|v| Json(v))
        .map_err(|e| match e {
            rusqlite::Error::QueryReturnedNoRows => Error {
                code: 404,
                message: "Not found".to_string(),
            },
            _ => Error {
                code: 500,
                message: "Internal server error".to_string(),
            },
        })
}
