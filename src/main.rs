mod controller;
mod db;
mod model;
mod provider;
mod repository;
mod service;
#[cfg(test)]
mod tests;

use crate::model::Error;
use color_eyre::Report;
use db::{Db, DbVersion};
use dotenv::dotenv;
use rocket::{catch, catchers, fairing::AdHoc, http::Status, routes, Build, Request, Rocket};
use std::path::Path;
use std::{env, process::exit};
use tracing::{error, warn};
use tracing_subscriber::EnvFilter;

#[rocket::main]
async fn main() -> Result<(), Report> {
    dotenv().ok();
    setup()?;
    let args: Vec<String> = env::args().collect();
    cli(&args[1..]).await;
    Ok(())
}

fn setup() -> Result<(), Report> {
    if env::var("RUST_LIB_BACKTRACE").is_err() {
        env::set_var("RUST_LIB_BACKTRACE", "1")
    }
    color_eyre::install()?;

    if env::var("RUST_LOG").is_err() {
        env::set_var("RUST_LOG", "info")
    }
    tracing_subscriber::fmt::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    if env::var("DATA_DIR").is_err() {
        let mut dir = dirs::home_dir().unwrap();
        dir.push(".pfd");
        env::set_var("DATA_DIR", dir.to_str().unwrap());
    }

    let data_dir = env::var("DATA_DIR").unwrap();
    let data_dir = Path::new(&data_dir);

    if !data_dir.exists() {
        warn!(data_dir = ?data_dir.to_str(), "Data dir does not exist, creating");
        std::fs::create_dir_all(data_dir).unwrap();
    } else {
        warn!(data_dir = ?data_dir.to_str(), "Set data dir");
    }

    Ok(())
}

async fn cli(args: &[String]) {
    let action = args.get(0).unwrap_or_else(|| {
        error!(?args, "Action is not specified");
        exit(1);
    });

    match action.as_str() {
        "db" => db::cli(&args[1..]).await,
        "serve" => prepare(rocket::build()).launch().await.unwrap(),
        _ => {
            error!(%action, ?args, "Unknown action");
            exit(1);
        }
    }
}

fn prepare(rocket: Rocket<Build>) -> Rocket<Build> {
    rocket
        .mount("/", routes![controller::exchange_rates::get])
        .attach(Db::fairing())
        .attach(AdHoc::on_ignite("Run migrations", run_migrations))
        .register("/", catchers![default_catcher])
}

async fn run_migrations(rocket: Rocket<Build>) -> Rocket<Build> {
    let conf = rocket.figment().clone();
    let db = Db::get_one(&rocket).await.unwrap();
    db.run(move |conn| {
        db::migrate(&conf, conn, DbVersion::Latest).unwrap_or_else(|e| {
            error!(%e, "Migration failed");
            exit(1);
        })
    })
    .await;
    rocket
}

#[catch(default)]
fn default_catcher(status: Status, _request: &Request) -> Error {
    Error::short(status)
}
