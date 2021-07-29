mod controller;
mod migrations;
mod model;
mod repository;
#[cfg(test)]
mod tests;

use rocket::{fairing::AdHoc, figment::Figment, routes, Build, Rocket};
use rocket_sync_db_pools::database;
use rusqlite::Connection;
use std::env;
use std::fs::remove_file;

#[database("main")]
pub struct Db(Connection);

#[rocket::main]
async fn main() {
    let args: Vec<String> = env::args().collect();
    let action = &args[1];

    match action.as_str() {
        "db" => {
            let action = &args[2];
            let conf = rocket::Config::figment();

            match action.as_str() {
                "drop" => {
                    println!("Dropping database...");
                    let db = conf.find_value("databases.main.url").unwrap();
                    let db = db.as_str().unwrap();
                    println!("Database URL: {:?}", db);
                    remove_file(db).unwrap();
                    println!("Database has been dropped");
                }
                "migrate" => {
                    println!("Migrating database...");
                    let mut conn = get_db_connection(&conf);
                    println!("Connected");
                    migrations::run(&mut conn);
                    println!("Database schema has been migrated");
                }
                _ => println!("Unknown action: {}", action),
            };
        }
        "serve" => prepare(rocket::build()).launch().await.unwrap(),
        _ => println!("Unknown action: {}", action),
    }
}

fn prepare(rocket: Rocket<Build>) -> Rocket<Build> {
    rocket
        .mount("/", routes![controller::exchange_rates::get])
        .attach(Db::fairing())
        .attach(AdHoc::on_ignite("Run migrations", run_migrations))
}

async fn run_migrations(rocket: Rocket<Build>) -> Rocket<Build> {
    let db = Db::get_one(&rocket).await.unwrap();
    db.run(|conn| migrations::run(conn)).await;
    rocket
}

fn get_db_connection(conf: &Figment) -> Connection {
    let url = conf.find_value("databases.main.url").unwrap();
    let url = url.as_str().unwrap();
    Connection::open(url).unwrap()
}
