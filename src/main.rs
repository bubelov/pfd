mod conf;
mod controller;
mod db;
mod model;
mod provider;
mod repository;
mod service;
#[cfg(test)]
mod tests;

use crate::model::ApiError;
use anyhow::Result;
use db::{Db, DbVersion};
use rocket::{
    catch, catchers, fairing::AdHoc, http::Status, routes, Build, Config, Request, Rocket,
};
use std::path::Path;
use std::{env, process::exit};
use tracing::{error, warn};
use tracing_log::AsLog;
use tracing_subscriber::fmt::Layer;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::EnvFilter;
use tracing_subscriber::Registry;

#[rocket::main]
async fn main() -> Result<()> {
    if env::var("DATA_DIR").is_err() {
        let dir = dirs::home_dir().unwrap().join(".pfd");
        env::set_var("DATA_DIR", dir.to_str().unwrap());
    }

    let data_dir = env::var("DATA_DIR")?;
    let data_dir = Path::new(&data_dir);

    if !data_dir.exists() {
        std::fs::create_dir_all(data_dir).unwrap();
    }

    if env::var("RUST_LOG").is_err() {
        env::set_var("RUST_LOG", "info")
    }

    //let subscriber = Subscriber::builder()
    //    .with_env_filter(EnvFilter::from_default_env())
    //    .finish();

    let log_file_appender = tracing_appender::rolling::never(data_dir, "pfd.log");
    let (log_file_appender, _log_file_guard) = tracing_appender::non_blocking(log_file_appender);

    let subscriber = Registry::default()
        .with(EnvFilter::from_default_env())
        .with(Layer::new().with_writer(std::io::stdout))
        .with(Layer::new().with_writer(log_file_appender));

    tracing::subscriber::set_global_default(subscriber)?;

    tracing_log::LogTracer::builder()
        .with_max_level(tracing_core::LevelFilter::current().as_log())
        .init()?;

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
    rocket
        .mount(
            "/",
            routes![controller::exchange_rates::get, controller::users::post],
        )
        .attach(Db::fairing())
        .attach(AdHoc::on_ignite("Run migrations", run_migrations))
        .register("/", catchers![default_catcher])
}

async fn run_migrations(rocket: Rocket<Build>) -> Rocket<Build> {
    let db = Db::get_one(&rocket).await.unwrap();
    db.run(move |conn| {
        db::migrate(conn, DbVersion::Latest).unwrap_or_else(|e| {
            error!(%e, "Migration failed");
            exit(1);
        })
    })
    .await;
    rocket
}

#[catch(default)]
fn default_catcher(status: Status, _request: &Request) -> ApiError {
    ApiError::short(status)
}
