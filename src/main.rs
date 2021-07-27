mod controller;
mod migrations;
mod model;
mod repository;
#[cfg(test)]
mod tests;

use rocket::{fairing::AdHoc, routes, Build, Rocket};
use rocket_sync_db_pools::database;
use rusqlite::Connection;

#[database("main")]
pub struct Db(Connection);

#[rocket::main]
async fn main() {
    rocket(rocket::build()).await.launch().await.unwrap();
}

async fn rocket(rocket: Rocket<Build>) -> Rocket<Build> {
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
