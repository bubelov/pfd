use rocket::{get, launch, routes, catch, catchers};
use rocket::http::Status;
use rocket::Request;
use rocket::serde::{Serialize, Deserialize, json::Json};

use rocket_sync_db_pools::database;

use rusqlite::{Connection, named_params};

#[database("main")]
struct Db(Connection);

#[derive(Debug, Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
struct ExchangeRate {
    base: String,
    quote: String,
    rate: f64,
}

#[derive(Debug, Serialize)]
#[serde(crate = "rocket::serde")]
struct ErrorResponseBody {
    code: u16,
    message: String,
}

#[get("/exchange_rates?<base>&<quote>", format = "json")]
async fn get_exchange_rate(
    base: &str,
    quote: &str,
    db: Db,
) -> Option<Json<ExchangeRate>> {
    let base = base.to_string();
    let quote = quote.to_string();
    let rate = db.run(move |c| {
        c.query_row(
            "SELECT rate FROM exchange_rate WHERE base = :base AND quote = :quote",
            named_params!{":base": &base, ":quote": &quote},
            |r| {
            Ok(ExchangeRate {
                base: base.clone(),
                quote: quote.clone(),
                rate: r.get(0)?
            })
        })
    }).await.ok()?;

    Some(Json(rate))
}

#[catch(default)]
fn error(status: Status, req: &Request) -> Json<ErrorResponseBody> {
    Json(
        ErrorResponseBody {
            code: status.code,
            message: format!("Failed to handle URI {}", req.uri())
        }
    )
}

#[launch]
fn rocket() -> _ {
    rocket::build()
        .mount("/", routes![get_exchange_rate])
        .register("/", catchers![error])
        .attach(Db::fairing())
}
