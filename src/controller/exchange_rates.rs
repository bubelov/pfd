use crate::{
    db::Db,
    model::{Error, ExchangeRate, User},
    service::exchange_rates,
};
use rocket::{get, http::Status, serde::json::Json};

#[get("/exchange_rates?<quote>&<base>")]
pub async fn get(
    quote: &str,
    base: &str,
    db: Db,
    _user: User,
) -> Result<Option<Json<ExchangeRate>>, Error> {
    exchange_rates::get_by_quote_and_base(quote, base, db)
        .await
        .map(|opt| opt.map(|v| Json(v)))
        .map_err(|e| Error::full(Status::InternalServerError, e))
}
