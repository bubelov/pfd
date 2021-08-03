use crate::{model::ExchangeRate, repository::exchange_rates};
use rocket::{figment::Figment, serde::Deserialize, Config};
use rocket_sync_db_pools::database;
use rusqlite::Connection;
use std::{
    error::Error,
    fs::remove_file,
    io::{copy, Cursor},
    process::exit,
};
use zip::ZipArchive;

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
enum FiatRatesProviderType {
    #[serde(rename = "ecb")]
    Ecb,
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
            let mut conn = connect(&conf);
            let version = match args.get(1) {
                Some(version) => DbVersion::Specific(version.parse::<i16>().unwrap()),
                None => DbVersion::Latest,
            };
            migrate(&conf, &mut conn, version);
        }
        "sync" => sync().await.unwrap_or_else(|e| {
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
            conn.execute(&migr.up, []).unwrap();
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
            conn.execute(&migr.down, []).unwrap();
            conn.execute(&format!("PRAGMA user_version={}", migr.version - 1), [])
                .unwrap();
        }
    }
}

async fn sync() -> Result<(), Box<dyn Error>> {
    let conf = Config::figment();
    let fiat_provider: FiatRatesProviderType = conf.extract_inner("providers.fiat.type")?;

    match fiat_provider {
        FiatRatesProviderType::Ecb => sync_ecb().await?,
    }

    Ok(())
}

async fn sync_ecb() -> Result<(), Box<dyn Error>> {
    let url = "https://www.ecb.europa.eu/stats/eurofxref/eurofxref.zip";
    let res = reqwest::get(url).await?;
    let body = Cursor::new(res.bytes().await?);
    let mut archive = ZipArchive::new(body)?;
    let mut compressed_csv = archive.by_index(0)?;
    let mut csv: Vec<u8> = vec![];
    copy(&mut compressed_csv, &mut csv)?;
    let csv = String::from_utf8(csv)?;
    let lines: Vec<&str> = csv.lines().collect();
    let headers: Vec<&str> = lines[0].strip_suffix(", ").unwrap().split(", ").collect();
    let codes = &headers[1..];
    let values: Vec<&str> = lines[1].strip_suffix(", ").unwrap().split(", ").collect();
    let rates = &values[1..];

    let rates: Vec<ExchangeRate> = codes
        .iter()
        .zip(rates.iter())
        .map(|(code, rate)| ExchangeRate {
            quote: code.to_string(),
            base: "EUR".to_string(),
            rate: 1.0 / rate.parse::<f64>().unwrap(),
        })
        .collect();

    let conf = Config::figment();
    let mut conn = connect(&conf);

    for rate in rates {
        exchange_rates::insert_or_replace(&mut conn, &rate);
    }

    Ok(())
}

pub fn connect(conf: &Figment) -> Connection {
    let url = conf.find_value("databases.main.url").unwrap();
    let url = url.as_str().unwrap();
    Connection::open(url).unwrap()
}

fn schema_version(conn: &Connection) -> rusqlite::Result<i16> {
    conn.query_row("SELECT user_version FROM pragma_user_version", [], |row| {
        row.get(0)
    })
}
