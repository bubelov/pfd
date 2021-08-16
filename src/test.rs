use crate::{
    db::migrate_to_latest,
    model::{AuthToken, User},
    prepare,
    repository::{AuthTokenRepository, UserRepository},
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
    let conn = Connection::open(&db_url).unwrap();
    let rocket = prepare(rocket::custom(&conf)).attach(AdHoc::on_request("Authorize", |req, _| {
        Box::pin(async move {
            req.add_header(Header::new(
                "AuthorizatioN",
                format!("Bearer {}", AUTH_TOKEN),
            ));
        })
    }));
    let client = Client::untracked(rocket).unwrap();
    let user = User {
        username: "test".to_string(),
        password_hash:
            "$argon2i$v=19$m=4096,t=3,p=1$dGVzdHNhbHQ$vZDbLe7RwrtwcAi3fktiLHdK3/PLogGcGuCgDpoINUc"
                .to_string(),
    };
    let token = AuthToken {
        id: AUTH_TOKEN.parse().unwrap(),
        username: user.username.clone(),
    };
    let user_repo = UserRepository::new(Connection::open(&db_url).unwrap());
    user_repo.insert(&user).unwrap();
    let token_repo = AuthTokenRepository::new(Connection::open(&db_url).unwrap());
    token_repo.insert(&token).unwrap();
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
