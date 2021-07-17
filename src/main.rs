use rocket::{get, launch, routes, catch, catchers};
use rocket::http::Status;
use rocket::Request;
use rocket::serde::{Serialize, Deserialize, json::Json};
use rocket::tokio::sync::Mutex;
use rocket::State;

use rocket_sync_db_pools::database;

use rusqlite::{named_params, Connection};

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
    ) -> Option<Json<ExchangeRate>> {
    let rate = ExchangeRate {
        base: base.to_string(),
        quote: quote.to_string(),
        rate: 35000.0
    };
    
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
    //println!("Setting up database");
    //let db = get_db().unwrap();

    rocket::build()
        .mount("/", routes![get_exchange_rates])
        .register("/", catchers![error])
        .attach(Db::fairing())
        //.manage(Db::new(db))
}

fn get_db() -> rusqlite::Result<Connection> {
    let db = Connection::open_in_memory()?;

    db.execute(
        "CREATE TABLE exchange_rate (base, quote, rate)",
        [],
    )?;

    let rate = ExchangeRate {
        base: "USD".to_string(),
        quote: "BTC".to_string(),
        rate: 35000.0,
    };

    let mut stmt = db.prepare("INSERT INTO exchange_rate VALUES (:base, :quote, :rate)")?;
    stmt.execute(named_params!{ ":base": rate.base, ":quote": rate.quote, ":rate": rate.rate })?;
    drop(stmt);

    Ok(db)
}
