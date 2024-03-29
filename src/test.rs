use crate::{
    attach_payload,
    conf::Conf,
    db::migrate_to_latest,
    model::{AuthToken, User},
    repository::{AuthTokenRepository, UserRepository},
};
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use rocket::{fairing::AdHoc, http::Header, local::blocking::Client};
use rusqlite::Connection;
use std::{
    env,
    sync::atomic::{AtomicUsize, Ordering},
};

static COUNTER: AtomicUsize = AtomicUsize::new(1);

pub fn client() -> Client {
    env::set_var("DATA_DIR", "");

    const AUTH_TOKEN: &str = "5110afcc-f3cc-420e-bb8c-a4f425af74c8";

    let db_name = COUNTER.fetch_add(1, Ordering::Relaxed);
    let db_url = format!("file::testdb_{}:?mode=memory&cache=shared", db_name);

    let conf = Conf::new().unwrap();
    let conf = Conf { db_url, ..conf };

    let rocket =
        attach_payload(rocket::build(), conf).attach(AdHoc::on_request("Authorize", |req, _| {
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

    let user_repo = client.rocket().state::<UserRepository>().unwrap();
    user_repo.insert(&user).unwrap();

    let token_repo = client.rocket().state::<AuthTokenRepository>().unwrap();
    token_repo
        .insert(&AuthToken {
            id: AUTH_TOKEN.parse().unwrap(),
            username: user.username.clone(),
        })
        .unwrap();

    client
}

pub fn pool() -> Pool<SqliteConnectionManager> {
    let db_name = COUNTER.fetch_add(1, Ordering::Relaxed);
    let db_url = format!("file::testdb_{}:?mode=memory&cache=shared", db_name);
    let mut conn = Connection::open(&db_url).unwrap();
    migrate_to_latest(&mut conn).unwrap();
    let manager = SqliteConnectionManager::file(&db_url);
    Pool::new(manager).unwrap()
}
