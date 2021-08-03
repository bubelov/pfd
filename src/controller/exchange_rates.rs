use crate::{
    db::Db,
    model::{Error, ExchangeRate},
    service::exchange_rates,
};
use rocket::{get, serde::json::Json};

#[get("/exchange_rates?<quote>&<base>")]
pub async fn get(quote: &str, base: &str, db: Db) -> Result<Json<ExchangeRate>, Error> {
    match exchange_rates::get(quote, base, db).await {
        Ok(v) => match v {
            Some(v) => Ok(Json(v)),
            None => Err(Error::new(404, "Not found")),
        },
        Err(e) => {
            eprintln!("{}", e);
            Err(Error::new(500, "Internal server error"))
        }
    }
}
