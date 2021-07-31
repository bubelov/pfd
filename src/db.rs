use rocket::{figment::Figment, serde::Deserialize, Config};
use rocket_sync_db_pools::database;
use rusqlite::Connection;
use std::fs::remove_file;

#[database("main")]
pub struct Db(Connection);

pub enum DbVersion {
    Specific(i16),
    Latest,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct Migration {
    version: i16,
    up: String,
    down: String,
}

#[derive(Debug, Deserialize)]
#[serde(crate = "rocket::serde")]
enum RatesProvider {
    #[serde(rename = "ecb")]
    Ecb,
}

pub fn cli(args: &Vec<String>) {
    let action = &args[2];
    let conf = Config::figment();

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
            let mut conn = connect(&conf);
            migrate(&conf, &mut conn, DbVersion::Latest);
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

pub fn migrate(conf: &Figment, conn: &mut Connection, target_version: DbVersion) {
    let current_version = schema_version(conn).unwrap();
    println!("Current schema version: {}", current_version);

    let migrations: Vec<Migration> = conf.extract_inner("migrations").unwrap();
    println!("Migrations found: {}", migrations.len());

    let target_version = match target_version {
        DbVersion::Latest => {
            migrations
                .iter()
                .max_by_key(|it| it.version)
                .unwrap()
                .version
        }
        DbVersion::Specific(v) => v,
    };

    println!("Target version: {}", target_version);

    if current_version == target_version {
        println!("Schema is up to date");
    } else if current_version < target_version {
        println!("Schema is outdated, updating...");
        let migrations: Vec<Migration> = migrations
            .iter()
            .filter(|it| it.version > current_version)
            .cloned()
            .collect();
        println!("Pending migrations found: {}", migrations.len());
        for migr in migrations {
            println!("Updating schema to version {}", migr.version);
            println!("{}", &migr.up);
            conn.execute(&migr.up, []).unwrap();
            conn.execute(&format!("PRAGMA user_version={}", migr.version), [])
                .unwrap();
        }
    } else {
        println!("Downgrading the schema...");
        let migrations: Vec<Migration> = migrations
            .iter()
            .filter(|it| it.version <= current_version)
            .cloned()
            .collect();
        println!("Pending migrations found: {}", migrations.len());
        for migr in migrations.iter().rev() {
            println!("Downgrading schema to version {}", migr.version - 1);
            println!("{}", &migr.down);
            conn.execute(&migr.down, []).unwrap();
            conn.execute(&format!("PRAGMA user_version={}", migr.version - 1), [])
                .unwrap();
        }
    }
}

fn schema_version(conn: &Connection) -> rusqlite::Result<i16> {
    conn.query_row("SELECT user_version FROM pragma_user_version", [], |row| {
        row.get(0)
    })
}
