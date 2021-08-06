use crate::{
    db::Db,
    model::{ApiResult, ExchangeRate, User},
    service::exchange_rates,
};
use rocket::get;

#[get("/exchange_rates?<quote>&<base>")]
pub async fn get(quote: &str, base: &str, db: Db, _user: User) -> ApiResult<ExchangeRate> {
    ApiResult::new(exchange_rates::get_by_quote_and_base(quote, base, db).await)
}
