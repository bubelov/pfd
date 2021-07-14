use rocket::{get, launch, routes, catch, catchers};
use rocket::response::status;
use rocket::http::Status;
use rocket::Request;

use rusqlite::{named_params, Connection};

#[derive(Debug)]
struct ExchangeRate {
    base: String,
    quote: String,
    rate: f64,
}

#[get("/exchange_rates")]
fn get_exchange_rates() -> &'static str {
    "[]"
}

#[catch(default)]
fn error(status: Status, req: &Request<'_>) -> status::Custom<String> {
    let msg = format!("{} ({})", status, req.uri());
    status::Custom(status, msg)
}

#[launch]
async fn rocket() -> _ {
    let conn = create_db();

    rocket::build()
        .mount("/", routes![get_exchange_rates])
        .register("/", catchers![error])
}

async fn create_db() -> rusqlite::Result<Connection> {
    let conn = Connection::open_in_memory()?;

    conn.execute(
        "CREATE TABLE exchange_rate (base, quote, rate)",
        [],
    )?;

    let rate = ExchangeRate {
        base: "USD".to_string(),
        quote: "BTC".to_string(),
        rate: 35000.0,
    };

    let mut stmt = conn.prepare("INSERT INTO exchange_rate VALUES (:base, :quote, :rate)")?;
    stmt.execute(named_params!{ ":base": rate.base, ":quote": rate.quote, ":rate": rate.rate })?;
    drop(stmt);

    Ok(conn)
}
