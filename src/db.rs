use crate::migrations;
use rocket::{figment::Figment, serde::Deserialize};
use rocket_sync_db_pools::database;
use rusqlite::Connection;
use std::fs::remove_file;

#[database("main")]
pub struct Db(Connection);

#[derive(Debug, Deserialize)]
#[serde(crate = "rocket::serde")]
enum RatesProvider {
    #[serde(rename = "ecb")]
    Ecb,
}

pub fn cli(args: Vec<String>) {
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
            let mut conn = connect(&conf);
            println!("Connected");
            migrations::run(&mut conn);
            println!("Database schema has been migrated");
        }
        "sync" => {
    let provider: RatesProvider = conf.extract_inner("provider").unwrap();
    println!("Provider: {:?}", provider);
        }
        _ => println!("Unknown action: {}", action),
    };
}

pub fn connect(conf: &Figment) -> Connection {
    let url = conf.find_value("databases.main.url").unwrap();
    let url = url.as_str().unwrap();
    Connection::open(url).unwrap()
}
