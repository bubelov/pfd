#[rocket::get("/exchange_rates?<base>&<quote>", format = "json")]
pub async fn get(
    base: &str,
    quote: &str,
    db: crate::Db,
) -> Option<rocket::serde::json::Json<crate::model::ExchangeRate>> {
    let base = base.to_string();
    let quote = quote.to_string();

    let rate = db.run(move |conn| {
        crate::repository::exchange_rates::find_by_base_and_quote(
            conn, 
            base, 
            quote,
        )
    }).await;

    rate.map(|rate| rocket::serde::json::Json(rate))
}
