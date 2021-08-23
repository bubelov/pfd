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
    conf::Conf,
    model::ApiError,
    repository::{AuthTokenRepository, ExchangeRateRepository, UserRepository},
    service::{AuthTokenService, ExchangeRateService, UserService},
};
use anyhow::Result;
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use rocket::{catch, catchers, fairing::AdHoc, http::Status, routes, Build, Request, Rocket};
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
    match args.len() {
        0 => attach_payload(rocket::build(), Conf::new().unwrap())
            .launch()
            .await
            .unwrap(),
        _ => match args.get(0).unwrap().as_str() {
            "db" => db::cli(&args[1..]).await,
            _ => {
                error!(?args, "Unknown action");
                exit(1);
            }
        },
    }
}

fn attach_payload(rocket: Rocket<Build>, conf: Conf) -> Rocket<Build> {
    let conn_manager = SqliteConnectionManager::file(conf.db_url);
    let pool = Pool::new(conn_manager).unwrap();

    let user_repo = UserRepository::new(&pool);
    let user_service = UserService::new(&user_repo);
    let token_repo = AuthTokenRepository::new(&pool);
    let token_service = AuthTokenService::new(&token_repo);
    let rate_repo = ExchangeRateRepository::new(&pool);
    let rate_service = ExchangeRateService::new(&rate_repo);

    rocket
        .manage(pool)
        .manage(user_repo)
        .manage(user_service)
        .manage(token_repo)
        .manage(token_service)
        .manage(rate_repo)
        .manage(rate_service)
        .attach(AdHoc::on_ignite("Run migrations", run_migrations))
        .register("/", catchers![default_catcher])
        .mount(
            "/",
            routes![
                controller::exchange_rate::get,
                controller::user::post,
                controller::auth_token::post
            ],
        )
}

async fn run_migrations(rocket: Rocket<Build>) -> Rocket<Build> {
    let pool: &Pool<SqliteConnectionManager> = rocket.state().unwrap();

    db::migrate_to_latest(&mut pool.get().unwrap()).unwrap_or_else(|e| {
        error!(%e, "Migration failed");
        exit(1);
    });

    rocket
}

#[catch(default)]
fn default_catcher(status: Status, _request: &Request) -> ApiError {
    status.into()
}
