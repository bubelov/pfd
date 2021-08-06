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
) -> Result<Json<ExchangeRate>, Error> {
    match exchange_rates::get(quote, base, db).await {
        Ok(v) => match v {
            Some(v) => Ok(Json(v)),
            None => Err(Error::short(Status::NotFound)),
        },
        Err(e) => Err(Error::full(Status::InternalServerError, e)),
    }
}
