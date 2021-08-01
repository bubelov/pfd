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

    match rate {
        Ok(rate) => match rate {
            Some(rate) => Ok(Json(rate)),
            None => {
                let base_owned = base.to_string();
                let quote_owned = quote.to_string();

                let inversed_rate = db
                    .run(move |conn| {
                        exchange_rates::select_by_base_and_quote(conn, &quote_owned, &base_owned)
                    })
                    .await;

                match inversed_rate {
                    Ok(rate) => match rate {
                        Some(rate) => Ok(Json(ExchangeRate {
                            base: base.to_string(),
                            quote: quote.to_string(),
                            rate: 1.0 / rate.rate,
                        })),
                        None => Err(Error {
                            code: 404,
                            message: "Not found".to_string(),
                        }),
                    },
                    Err(_e) => Err(Error {
                        code: 500,
                        message: "Internal server error".to_string(),
                    }),
                }
            }
        },
        Err(_e) => Err(Error {
            code: 500,
            message: "Internal server error".to_string(),
        }),
    }
}
