mod conf;
mod controller;
mod db;
mod model;
mod provider;
mod repository;
mod service;
#[cfg(test)]
mod test;

use crate::{
    model::ApiError,
    repository::{AuthTokenRepository, ExchangeRateRepository, UserRepository},
};
use anyhow::Result;
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use rocket::{
    catch, catchers, fairing::AdHoc, http::Status, routes, Build, Config, Request, Rocket,
};
use rusqlite::Connection;
use std::{env, path::Path, process::exit};
use tracing::{error, warn};

#[rocket::main]
async fn main() -> Result<()> {
    if env::var("RUST_BACKTRACE").is_err() {
        env::set_var("RUST_BACKTRACE", "1");
    }

    if env::var("RUST_LOG").is_err() {
        env::set_var("RUST_LOG", "info")
    }

    tracing_subscriber::fmt::init();

    if env::var("DATA_DIR").is_err() {
        let dir = dirs::home_dir()
            .unwrap_or_else(|| {
                error!("Can't find home directory");
                exit(1);
            })
            .join(".pfd");

        env::set_var("DATA_DIR", dir.to_str().unwrap());
    }

    let data_dir = env::var("DATA_DIR")?;
    let data_dir = Path::new(&data_dir);

    if !data_dir.exists() {
        std::fs::create_dir_all(data_dir).unwrap();
    }

    let args: Vec<String> = env::args().collect();
    warn!("Starting up");
    warn!(data_dir = %data_dir.display());
    warn!(?args);
    cli(&args[1..]).await;
    Ok(())
}

async fn cli(args: &[String]) {
    let db_url = env::var("DATA_DIR").unwrap();
    let db_url = Path::new(&db_url).join("pfd.db");

    let conf = Config::figment()
        .merge(("databases.main.url", db_url.to_str()))
        .merge(("cli_colors", false))
        .merge(("log_level", "off"));

    match args.len() {
        0 => prepare(rocket::custom(conf)).launch().await.unwrap(),
        _ => match args.get(0).unwrap().as_str() {
            "db" => db::cli(&args[1..], &conf).await,
            _ => {
                error!(?args, "Unknown action");
                exit(1);
            }
        },
    }
}

fn prepare(rocket: Rocket<Build>) -> Rocket<Build> {
    let db_url = rocket.figment().find_value("databases.main.url").unwrap();
    let db_url = db_url.as_str().unwrap();
    warn!(?db_url);

    let conn_manager = SqliteConnectionManager::file(db_url);
    let pool = Pool::new(conn_manager).unwrap();

    let user_repo = UserRepository::new(pool.clone());
    let token_repo = AuthTokenRepository::new(pool.clone());
    let rate_repo = ExchangeRateRepository::new(pool.clone());

    rocket
        .mount(
            "/",
            routes![
                controller::exchange_rate::get,
                controller::user::post,
                controller::auth_token::post
            ],
        )
        .attach(AdHoc::on_ignite("Run migrations", run_migrations))
        .manage(user_repo)
        .manage(token_repo)
        .manage(rate_repo)
        .register("/", catchers![default_catcher])
}

async fn run_migrations(rocket: Rocket<Build>) -> Rocket<Build> {
    let db_url = rocket.figment().find_value("databases.main.url").unwrap();
    let db_url = db_url.as_str().unwrap();
    let mut conn = Connection::open(db_url).unwrap();

    db::migrate_to_latest(&mut conn).unwrap_or_else(|e| {
        error!(%e, "Migration failed");
        exit(1);
    });

    rocket
}

#[catch(default)]
fn default_catcher(status: Status, _request: &Request) -> ApiError {
    status.into()
}
