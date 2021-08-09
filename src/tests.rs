use crate::{
    model::{AuthToken, ExchangeRate, User},
    prepare,
    repository::{auth_tokens, exchange_rates, users},
};
use rocket::{
    fairing::AdHoc,
    http::{Header, Status},
    local::blocking::Client,
};
use rusqlite::Connection;
use std::sync::atomic::{AtomicUsize, Ordering};

static COUNTER: AtomicUsize = AtomicUsize::new(1);

static USER_ID: &str = "9b91bff6-b09c-4d7a-bf63-2aac76793b35";
static AUTH_TOKEN: &str = "5110afcc-f3cc-420e-bb8c-a4f425af74c8";

fn setup() -> (Client, Connection) {
    let db_name = COUNTER.fetch_add(1, Ordering::Relaxed);
    let db_url = format!("file::testdb_{}:?mode=memory&cache=shared", db_name);
    let conf = rocket::Config::figment().merge(("databases.main.url", &db_url));
    let mut conn = Connection::open(&db_url).unwrap();
    let rocket = prepare(rocket::custom(&conf)).attach(AdHoc::on_request("Authorize", |req, _| {
        Box::pin(async move {
            req.add_header(Header::new(
                "Authorization",
                format!("Bearer {}", AUTH_TOKEN),
            ));
        })
    }));
    let client = Client::untracked(rocket).unwrap();
    let user = User {
        id: USER_ID.parse().unwrap(),
    };
    let token = AuthToken {
        id: AUTH_TOKEN.parse().unwrap(),
        user_id: user.id.clone(),
    };
    users::insert_or_replace(&user, &mut conn).unwrap();
    auth_tokens::insert_or_replace(&token, &mut conn).unwrap();
    (client, conn)
}

fn setup_without_auth() -> Client {
    let db_name = COUNTER.fetch_add(1, Ordering::Relaxed);
    let db_url = format!("file::testdb_{}:?mode=memory&cache=shared", db_name);
    let conf = rocket::Config::figment().merge(("databases.main.url", &db_url));
    let rocket = prepare(rocket::custom(&conf));
    let client = Client::untracked(rocket).unwrap();
    client
}

#[test]
fn users_controller_post() {
    let client = setup_without_auth();
    let res = client.post("/users").dispatch();
    assert_eq!(res.status(), Status::Created);
    use crate::controller::users::PostPayload;
    res.into_json::<PostPayload>().unwrap();
}

#[test]
fn exchange_rates_controller_get_unauthorized() {
    let client = setup_without_auth();
    let res = client.get("/exchange_rates?quote=EUR&base=USD").dispatch();
    assert_eq!(res.status(), Status::Unauthorized);
}

#[test]
fn exchange_rates_controller_get() {
    let (client, mut db) = setup();

    let rate = ExchangeRate {
        quote: "EUR".to_string(),
        base: "USD".to_string(),
        rate: 1.25,
    };

    exchange_rates::insert_or_replace(&mut db, &rate).unwrap();

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

    exchange_rates::insert_or_replace(&mut db, &rate).unwrap();

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

    exchange_rates::insert_or_replace(&mut db, &usd_eur).unwrap();
    exchange_rates::insert_or_replace(&mut db, &rub_eur).unwrap();

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

#[test]
fn exchange_rates_controller_get_should_return_500_if_sql_query_fails() {
    let (client, db) = setup();
    db.execute_batch("DROP TABLE exchange_rate").unwrap();
    let res = client.get("/exchange_rates?quote=EUR&base=USD").dispatch();
    assert_eq!(res.status(), Status::InternalServerError);
}
