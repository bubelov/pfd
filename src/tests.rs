use crate::{
    model::ExchangeRate,
    prepare,
    repository::exchange_rates,
    {db, db::DbVersion},
};
use rocket::{http::Status, local::blocking::Client};
use rusqlite::Connection;
use serial_test::serial;

fn setup() -> (Client, Connection) {
    let conf = rocket::Config::figment().select("test");
    let rocket = prepare(rocket::custom(&conf));
    let client = Client::untracked(rocket).unwrap();
    (client, db::connect(&conf))
}

fn drop_db() {
    let conf = rocket::Config::figment().select("test");
    let mut conn = db::connect(&conf);
    db::migrate(&conf, &mut conn, DbVersion::Specific(0));
}

#[test]
#[serial]
fn exchange_rates_controller_get() {
    let (client, mut db) = setup();

    let rate = ExchangeRate {
        base: "USD".to_string(),
        quote: "EUR".to_string(),
        rate: 1.25,
    };

    exchange_rates::insert(&mut db, &rate);

    let res = client.get("/exchange_rates?base=USD&quote=EUR").dispatch();

    assert_eq!(res.status(), Status::Ok);
    let body = res.into_json::<ExchangeRate>().unwrap();
    assert_eq!(rate, body);

    drop_db();
}

#[test]
#[serial]
fn exchange_rates_controller_get_should_return_404_if_not_found() {
    let (client, _) = setup();
    let res = client.get("/exchange_rates?base=USD&quote=EUR").dispatch();
    assert_eq!(res.status(), Status::NotFound);
    drop_db();
}
