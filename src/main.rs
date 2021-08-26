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
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use rocket::{catch, catchers, fairing::AdHoc, http::Status, routes, Build, Request, Rocket};
use std::{
    env::{self, VarError},
    path::Path,
    process::exit,
};
use tracing::{error, warn};
use tracing_subscriber::EnvFilter;

#[rocket::main]
async fn main() {
    if env::var("RUST_BACKTRACE").is_err() {
        env::set_var("RUST_BACKTRACE", "1");
    }

    if env::var("RUST_LOG").is_err() {
        env::set_var("RUST_LOG", "info");
    }

    tracing_subscriber::fmt::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    let args: Vec<String> = env::args().collect();
    warn!(data_dir = ?init_data_dir(), ?args, "Starting up");
    cli(&args[1..]).await;
}

async fn cli(args: &[String]) {
    match args.len() {
        0 => {
            let conf = Conf::new().unwrap_or_else(|e| {
                error!(?e, "Failed to read configuration");
                exit(1);
            });

            let server: Result<(), rocket::Error> =
                attach_payload(rocket::build(), conf).launch().await;

            if let Err(e) = server {
                error!(?e, "Failed to start a server");
                exit(1);
            }
        }
        _ => match args.first().unwrap().as_str() {
            "db" => db::cli(&args[1..]).await.unwrap_or_else(|e| {
                error!("{}", e.to_string());
                exit(1);
            }),
            _ => {
                error!("Unknown action");
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

fn init_data_dir() -> String {
    match env::var("DATA_DIR") {
        Ok(data_dir) => {
            if Path::new(&data_dir).exists() {
                data_dir
            } else {
                error!(?data_dir, "Data dir doesn't exist");
                exit(1);
            }
        }
        Err(e) => match e {
            VarError::NotPresent => {
                let data_dir = dirs::home_dir()
                    .unwrap_or_else(|| {
                        error!("Can't find home directory");
                        exit(1);
                    })
                    .join(".pfd");

                if !data_dir.exists() {
                    std::fs::create_dir_all(&data_dir).unwrap_or_else(|e| {
                        error!(?e, ?data_dir, "Failed to create data dir");
                        exit(1);
                    });
                }

                let data_dir = data_dir.to_str().unwrap_or_else(|| {
                    error!("Home dir path is not unicode");
                    exit(1);
                });

                env::set_var("DATA_DIR", data_dir);
                data_dir.into()
            }
            VarError::NotUnicode(ref os_string) => {
                error!(%e, ?os_string, "DATA_DIR is not unicode");
                exit(1);
            }
        },
    }
}
