use crate::{
    db::migrate_to_latest,
    model::{AuthToken, User},
    prepare,
    repository::{auth_token, user},
};
use rocket::{fairing::AdHoc, http::Header, local::blocking::Client};
use rusqlite::Connection;
use std::sync::atomic::{AtomicUsize, Ordering};

static COUNTER: AtomicUsize = AtomicUsize::new(1);

static AUTH_TOKEN: &str = "5110afcc-f3cc-420e-bb8c-a4f425af74c8";

pub fn setup() -> (Client, Connection) {
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
        username: "test".to_string(),
        password_hash: "test".to_string(),
    };
    let token = AuthToken {
        id: AUTH_TOKEN.parse().unwrap(),
        username: user.username.clone(),
    };
    user::insert_or_replace(&user, &mut conn).unwrap();
    auth_token::insert_or_replace(&token, &mut conn).unwrap();
    (client, conn)
}

pub fn setup_without_auth() -> (Client, Connection) {
    let db_name = COUNTER.fetch_add(1, Ordering::Relaxed);
    let db_url = format!("file::testdb_{}:?mode=memory&cache=shared", db_name);
    let conf = rocket::Config::figment().merge(("databases.main.url", &db_url));
    let rocket = prepare(rocket::custom(&conf));
    let client = Client::untracked(rocket).unwrap();
    let conn = Connection::open(&db_url).unwrap();
    (client, conn)
}

pub fn setup_db() -> Connection {
    let db_name = COUNTER.fetch_add(1, Ordering::Relaxed);
    let db_url = format!("file::testdb_{}:?mode=memory&cache=shared", db_name);
    let mut conn = Connection::open(&db_url).unwrap();
    migrate_to_latest(&mut conn).unwrap();
    conn
}
