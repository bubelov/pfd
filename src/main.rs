use rocket::{get, launch, routes, catch, catchers};
use rocket::http::Status;
use rocket::Request;
use rocket::serde::json::{json, Value};

use rusqlite::{named_params, Connection};

#[derive(Debug)]
struct ExchangeRate {
    base: String,
    quote: String,
    rate: f64,
}

#[get("/exchange_rates", format = "json")]
fn get_exchange_rates() -> &'static str {
    "[]"
}

#[catch(default)]
fn error(status: Status, req: &Request<'_>) -> Value {
    json!({
        "code": status.code,
        "message": format!("Failed to handle URI {}", req.uri())
    })
}

#[launch]
fn rocket() -> _ {
    println!("Setting up database");
    get_db().unwrap();

    rocket::build()
        .mount("/", routes![get_exchange_rates])
        .register("/", catchers![error])
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
