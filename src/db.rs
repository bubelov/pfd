use crate::provider::{EcbFiatProvider, IexCryptoProvider};
use futures::join;
use rocket::{figment::Figment, serde::Deserialize, Config};
use rocket_sync_db_pools::database;
use rusqlite::Connection;
use std::{error::Error, fs::remove_file, process::exit};

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

pub async fn cli(args: &[String]) {
    let action = args.get(0).unwrap_or_else(|| {
        println!("Database action is not specified");
        exit(1);
    });

    match action.as_str() {
        "drop" => drop(),
        "migrate" => {
            let conf = Config::figment();
            let mut conn = connect();
            let version = match args.get(1) {
                Some(version) => DbVersion::Specific(version.parse::<i16>().unwrap()),
                None => DbVersion::Latest,
            };
            migrate(&conf, &mut conn, version);
        }
        "sync" => sync(&args[1..]).await.unwrap_or_else(|e| {
            println!("Sync failed with error: {:?}", e);
        }),
        _ => {
            println!("Unknown action: {}", action);
            exit(1);
        }
    };
}

fn drop() {
    println!("Dropping database...");
    let db = Config::figment().find_value("databases.main.url").unwrap();
    let db = db.as_str().unwrap();
    println!("Database URL: {:?}", db);
    remove_file(db).unwrap();
    println!("Database has been dropped");
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
            println!("{}", &migr.up.trim());
            conn.execute_batch(&migr.up).unwrap();
            conn.execute(&format!("PRAGMA user_version={}", migr.version), [])
                .unwrap();
        }
    } else {
        println!("Downgrading the schema...");
        let migrations: Vec<Migration> = migrations
            .iter()
            .filter(|it| it.version > target_version)
            .cloned()
            .collect();
        println!("Pending migrations found: {}", migrations.len());
        for migr in migrations.iter().rev() {
            println!(
                "Downgrading schema version from {} to {}",
                migr.version,
                migr.version - 1
            );
            println!("{}", &migr.down.trim());
            conn.execute_batch(&migr.down).unwrap();
            conn.execute(&format!("PRAGMA user_version={}", migr.version - 1), [])
                .unwrap();
        }
    }
}

async fn sync(args: &[String]) -> Result<(), Box<dyn Error>> {
    let default_target = "all".to_string();
    let target = args.get(0).unwrap_or(&default_target);
    let conf = Config::figment();

    let mut ecb_fiat = EcbFiatProvider::new(&conf, connect());
    let mut iex_crypto = IexCryptoProvider::new(&conf, connect());

    match target.as_str() {
        "schedule" => {
            join!(ecb_fiat.schedule(), iex_crypto.schedule());
            Ok(())
        }
        "all" => {
            if ecb_fiat.conf.enabled {
                ecb_fiat.sync().await?;
            }

            if iex_crypto.conf.enabled {
                iex_crypto.sync().await?;
            }

            Ok(())
        }
        "fiat" => {
            if ecb_fiat.conf.enabled {
                ecb_fiat.sync().await?;
            }

            Ok(())
        }
        "crypto" => {
            if iex_crypto.conf.enabled {
                iex_crypto.sync().await?;
            }

            Ok(())
        }
        _ => Err(Box::new(std::io::Error::new(
            std::io::ErrorKind::Other,
            "Unknown sync target",
        ))),
    }
}

pub fn connect() -> Connection {
    let conf = Config::figment();
    let url = conf.find_value("databases.main.url").unwrap();
    let url = url.as_str().unwrap();
    Connection::open(url).unwrap()
}

fn schema_version(conn: &Connection) -> rusqlite::Result<i16> {
    conn.query_row("SELECT user_version FROM pragma_user_version", [], |row| {
        row.get(0)
    })
}
