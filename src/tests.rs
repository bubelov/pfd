use crate::{db, model::ExchangeRate, prepare, repository::exchange_rates};
use rocket::{http::Status, local::blocking::Client};
use rusqlite::Connection;
use std::sync::atomic::{AtomicUsize, Ordering};

static COUNTER: AtomicUsize = AtomicUsize::new(1);

fn setup() -> (Client, Connection) {
    let db_name = COUNTER.fetch_add(1, Ordering::Relaxed);
    let db_url = format!("file::testdb_{}:?mode=memory&cache=shared", db_name);
    let conf = rocket::Config::figment().merge(("databases.main.url", db_url));
    let rocket = prepare(rocket::custom(&conf));
    let client = Client::untracked(rocket).unwrap();
    (client, db::connect(&conf))
}

#[test]
fn exchange_rates_controller_get() {
    let (client, mut db) = setup();

    let rate = ExchangeRate {
        base: "USD".to_string(),
        quote: "EUR".to_string(),
        rate: 1.25,
    };

    exchange_rates::insert_or_replace(&mut db, &rate);

    let res = client.get("/exchange_rates?base=USD&quote=EUR").dispatch();

    assert_eq!(res.status(), Status::Ok);
    let body = res.into_json::<ExchangeRate>().unwrap();
    assert_eq!(rate, body);
}

#[test]
fn exchange_rates_controller_get_should_return_404_if_not_found() {
    let (client, _) = setup();
    let res = client.get("/exchange_rates?base=USD&quote=EUR").dispatch();
    assert_eq!(res.status(), Status::NotFound);
}
