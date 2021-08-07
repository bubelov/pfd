use crate::provider::{EcbFiatProvider, IexCryptoProvider};
use color_eyre::Report;
use futures::join;
use rocket::{figment::Figment, serde::Deserialize};
use rocket_sync_db_pools::database;
use rusqlite::Connection;
use std::{fs::remove_file, process::exit};
use tracing::{error, info, warn};

#[database("main")]
pub struct Db(Connection);

#[derive(Debug)]
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

pub async fn cli(args: &[String], conf: &Figment) {
    let action = args.get(0).unwrap_or_else(|| {
        error!(?args, "Database action is not specified");
        exit(1);
    });

    match action.as_str() {
        "drop" => drop(conf).unwrap_or_else(|e| {
            error!(%e, "Unable drop database");
            exit(1);
        }),
        "migrate" => {
            let mut conn = connect(conf).unwrap_or_else(|e| {
                error!(%e, "Can't connect to database");
                exit(1);
            });
            let version = match args.get(1) {
                Some(version) => DbVersion::Specific(version.parse::<i16>().unwrap()),
                None => DbVersion::Latest,
            };
            migrate(conf, &mut conn, version).unwrap_or_else(|e| {
                error!(%e, "Migration failed");
                exit(1);
            });
        }
        "sync" => sync(&args[1..], conf).await.unwrap_or_else(|e| {
            error!(%e, "Sync failed");
            exit(1);
        }),
        _ => {
            error!(%action, ?args, "Unknown action");
            exit(1);
        }
    };
}

fn drop(conf: &Figment) -> Result<(), Report> {
    info!("Dropping database...");
    let path = conf.find_value("databases.main.url")?;
    let path = path.as_str().ok_or(std::io::Error::new(
        std::io::ErrorKind::Other,
        "Invalid database path",
    ))?;
    info!(%path, "Found db path");
    remove_file(path)?;
    info!("Database has been dropped");
    Ok(())
}

pub fn migrate(
    conf: &Figment,
    conn: &mut Connection,
    target_version: DbVersion,
) -> Result<(), Report> {
    let current_version = schema_version(conn)?;
    info!(?current_version, ?target_version, "Migrating db schema");

    let migrations: Vec<Migration> = conf.extract_inner("migrations")?;
    info!(count = migrations.len(), "Loaded migrations");

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

    if current_version == target_version {
        info!("Schema is up to date");
    } else if current_version < target_version {
        info!("Schema is outdated, updating...");
        let migrations: Vec<Migration> = migrations
            .iter()
            .filter(|it| it.version > current_version)
            .cloned()
            .collect();
        warn!(count = migrations.len(), "Found pending migrations");
        for migr in migrations {
            info!(%migr.version, sql = &migr.up.trim(), "Updating schema");
            conn.execute_batch(&migr.up)?;
            conn.execute_batch(&format!("PRAGMA user_version={}", migr.version))?;
        }
    } else {
        info!("Downgrading the schema...");
        let migrations: Vec<Migration> = migrations
            .iter()
            .filter(|it| it.version > target_version)
            .cloned()
            .collect();
        warn!(count = migrations.len(), "Found pending migrations");
        for migr in migrations.iter().rev() {
            info!(
                from = migr.version,
                to = migr.version - 1,
                sql = &migr.down.trim(),
                "Downgrading schema"
            );
            conn.execute_batch(&migr.down)?;
            conn.execute_batch(&format!("PRAGMA user_version={}", migr.version - 1))?;
        }
    }

    Ok(())
}

async fn sync(args: &[String], conf: &Figment) -> Result<(), Report> {
    let default_target = "all".to_string();
    let target = args.get(0).unwrap_or(&default_target);

    let mut ecb_fiat = EcbFiatProvider::new(&conf, connect(&conf)?)?;
    let mut iex_crypto = IexCryptoProvider::new(&conf, connect(&conf)?)?;

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
        _ => Err(Report::new(std::io::Error::new(
            std::io::ErrorKind::Other,
            "Unknown sync target",
        ))),
    }
}

pub fn connect(conf: &Figment) -> Result<Connection, Report> {
    let path = conf.find_value("databases.main.url")?;
    let path = path.as_str().ok_or(std::io::Error::new(
        std::io::ErrorKind::Other,
        "Invalid database path",
    ))?;
    Ok(Connection::open(path)?)
}

fn schema_version(conn: &Connection) -> rusqlite::Result<i16> {
    conn.query_row("SELECT user_version FROM pragma_user_version", [], |row| {
        row.get(0)
    })
}
