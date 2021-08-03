mod controller;
mod db;
mod model;
mod repository;
mod service;
#[cfg(test)]
mod tests;

use db::{Db, DbVersion};
use rocket::{fairing::AdHoc, routes, Build, Rocket};
use std::{env, process::exit};

#[rocket::main]
async fn main() {
    let args: Vec<String> = env::args().collect();
    cli(&args[1..]).await;
}

async fn cli(args: &[String]) {
    let action = args.get(0).unwrap_or_else(|| {
        println!("Action is not specified");
        exit(1);
    });

    match action.as_str() {
        "db" => db::cli(&args[1..]).await,
        "serve" => prepare(rocket::build()).launch().await.unwrap(),
        _ => {
            println!("Unknown action: {}", action);
            exit(1);
        }
    }
}

fn prepare(rocket: Rocket<Build>) -> Rocket<Build> {
    rocket
        .mount("/", routes![controller::exchange_rates::get])
        .attach(Db::fairing())
        .attach(AdHoc::on_ignite("Run migrations", run_migrations))
}

async fn run_migrations(rocket: Rocket<Build>) -> Rocket<Build> {
    let conf = rocket.figment().clone();
    let db = Db::get_one(&rocket).await.unwrap();
    db.run(move |conn| db::migrate(&conf, conn, DbVersion::Latest))
        .await;
    rocket
}
