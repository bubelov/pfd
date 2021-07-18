use rocket::{get, launch, routes, catch, catchers};
use rocket::http::Status;
use rocket::Request;
use rocket::serde::{Serialize, Deserialize, json::Json};
use rocket::response::Debug;

use rocket_sync_db_pools::database;

use rusqlite::Connection;

type Result<T, E = Debug<rusqlite::Error>> = std::result::Result<T, E>;

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
fn get_exchange_rates(
    base: &str,
    quote: &str,
    _db: Db,
    ) -> Result<Json<ExchangeRate>> {
    let rate = ExchangeRate {
        base: base.to_string(),
        quote: quote.to_string(),
        rate: 35000.0
    };
    
    Ok(Json(rate))
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
        .mount("/", routes![get_exchange_rates])
        .register("/", catchers![error])
        .attach(Db::fairing())
}
