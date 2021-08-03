use crate::{model::ExchangeRate, prepare, repository::exchange_rates};
use rocket::{http::Status, local::blocking::Client};
use rusqlite::Connection;
use std::sync::atomic::{AtomicUsize, Ordering};

static COUNTER: AtomicUsize = AtomicUsize::new(1);

fn setup() -> (Client, Connection) {
    let db_name = COUNTER.fetch_add(1, Ordering::Relaxed);
    let db_url = format!("file::testdb_{}:?mode=memory&cache=shared", db_name);
    let conf = rocket::Config::figment().merge(("databases.main.url", &db_url));
    let rocket = prepare(rocket::custom(&conf));
    let client = Client::untracked(rocket).unwrap();
    (client, Connection::open(db_url).unwrap())
}

#[test]
fn exchange_rates_controller_get() {
    let (client, mut db) = setup();

    let rate = ExchangeRate {
        quote: "EUR".to_string(),
        base: "USD".to_string(),
        rate: 1.25,
    };

    exchange_rates::insert_or_replace(&mut db, &rate);

    let res = client.get("/exchange_rates?quote=EUR&base=USD").dispatch();

    assert_eq!(res.status(), Status::Ok);
    let body = res.into_json::<ExchangeRate>().unwrap();
    assert_eq!(rate, body);
}

#[test]
fn exchange_rates_controller_get_inversed() {
    let (client, mut db) = setup();

    let rate = ExchangeRate {
        quote: "EUR".to_string(),
        base: "USD".to_string(),
        rate: 1.19,
    };

    let inversed_rate = ExchangeRate {
        quote: "USD".to_string(),
        base: "EUR".to_string(),
        rate: 1.0 / 1.19,
    };

    exchange_rates::insert_or_replace(&mut db, &rate);

    let res = client.get("/exchange_rates?quote=USD&base=EUR").dispatch();

    assert_eq!(res.status(), Status::Ok);
    let body = res.into_json::<ExchangeRate>().unwrap();
    assert_eq!(inversed_rate, body);
}

#[test]
fn exchange_rates_controller_get_indirect() {
    let (client, mut db) = setup();

    let usd_eur = ExchangeRate {
        quote: "USD".to_string(),
        base: "EUR".to_string(),
        rate: 0.840972163821378,
    };

    let rub_eur = ExchangeRate {
        quote: "RUB".to_string(),
        base: "EUR".to_string(),
        rate: 0.0115324823898994,
    };

    exchange_rates::insert_or_replace(&mut db, &usd_eur);
    exchange_rates::insert_or_replace(&mut db, &rub_eur);

    let rub_usd = ExchangeRate {
        quote: "RUB".to_string(),
        base: "USD".to_string(),
        rate: 0.0115324823898994 / 0.840972163821378,
    };

    let res = client.get("/exchange_rates?quote=RUB&base=USD").dispatch();
    assert_eq!(res.status(), Status::Ok);
    let body = res.into_json::<ExchangeRate>().unwrap();
    assert_eq!(rub_usd, body);

    let usd_rub = ExchangeRate {
        quote: "USD".to_string(),
        base: "RUB".to_string(),
        rate: 0.840972163821378 / 0.0115324823898994,
    };

    let res = client.get("/exchange_rates?quote=USD&base=RUB").dispatch();
    assert_eq!(res.status(), Status::Ok);
    let body = res.into_json::<ExchangeRate>().unwrap();
    assert_eq!(usd_rub, body);
}

#[test]
fn exchange_rates_controller_get_should_return_404_if_not_found() {
    let (client, _) = setup();
    let res = client.get("/exchange_rates?quote=EUR&base=USD").dispatch();
    assert_eq!(res.status(), Status::NotFound);
}
