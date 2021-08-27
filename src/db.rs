use crate::{
    conf::{Conf, Migration},
    provider::{Ecb, Iex, Provider},
    repository::ExchangeRateRepository,
};
use anyhow::{Context, Error, Result};
use futures::future::join_all;
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::Connection;
use std::{fs::remove_file, path::Path, process::exit};
use tracing::{error, info, warn};

#[derive(Debug)]
pub enum DbVersion {
    Specific(i16),
    Latest,
}

pub async fn cli(args: &[String]) -> Result<()> {
    let first_arg = args.first().ok_or(Error::msg("No args provided"))?;

    match first_arg.as_str() {
        "drop" => drop().context("Unable to drop database")?,
        "migrate" => {
            let version = match args.get(1) {
                Some(version) => DbVersion::Specific(version.parse::<i16>().unwrap()),
                None => DbVersion::Latest,
            };
            let pool = pool().unwrap();
            migrate(&mut pool.get().unwrap(), version).unwrap_or_else(|e| {
                error!(%e, "Migration failed");
                exit(1);
            });
        }
        "sync" => sync(&args[1..]).await.unwrap_or_else(|e| {
            error!(%e, "Sync failed");
            exit(1);
        }),
        _ => {
            error!(?args, "Unknown argument");
            exit(1);
        }
    };

    Ok(())
}

fn drop() -> Result<()> {
    let db_url: String = Conf::new()?.db_url;
    let db_url: &Path = Path::new(&db_url);
    warn!(?db_url, "Dropping database");

    if !db_url.exists() {
        warn!("Database doesn't exist");
    } else {
        remove_file(db_url)?;
        warn!("Database has been dropped");
    }

    Ok(())
}

pub fn migrate_to_latest(conn: &mut Connection) -> Result<()> {
    migrate(conn, DbVersion::Latest)
}

pub fn migrate(conn: &mut Connection, target_version: DbVersion) -> Result<()> {
    let current_version = schema_version(conn)?;
    info!(?current_version, ?target_version, "Migrating db schema");

    let migrations = Conf::new()?.migrations;
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
        warn!("Schema is outdated, updating...");
        let migrations: Vec<Migration> = migrations
            .iter()
            .filter(|it| it.version > current_version)
            .cloned()
            .collect();
        warn!(count = migrations.len(), "Found pending migrations");
        for migr in migrations {
            warn!(%migr.version, sql = &migr.up.trim(), "Updating schema");
            conn.execute_batch(&migr.up)?;
            conn.execute_batch(&format!("PRAGMA user_version={}", migr.version))?;
        }
    } else {
        warn!("Downgrading the schema...");
        let migrations: Vec<Migration> = migrations
            .iter()
            .filter(|it| it.version > target_version)
            .cloned()
            .collect();
        warn!(count = migrations.len(), "Found pending migrations");
        for migr in migrations.iter().rev() {
            warn!(
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

async fn sync(args: &[String]) -> Result<()> {
    let conf = Conf::new()?;
    let pool = pool()?;

    let ecb = Ecb::new(conf.providers.ecb, ExchangeRateRepository::new(&pool));
    let iex = Iex::new(conf.providers.iex, ExchangeRateRepository::new(&pool));

    match args.len() {
        0 => {
            let results = join_all(vec![ecb.schedule(), iex.schedule()]).await;

            for result in results {
                result?;
            }
        }
        1 => {
            if args.get(0).unwrap_or(&"".to_string()) == "now" {
                let results = join_all(vec![ecb.sync(), iex.sync()]).await;

                for result in results {
                    result?;
                }
            } else {
                error!(?args, "Invalid arguments");
                exit(1);
            }
        }
        _ => error!(?args, "Invalid arguments"),
    }

    Ok(())
}

pub fn pool() -> Result<Pool<SqliteConnectionManager>> {
    let db_url = Conf::new()?.db_url;
    let manager = SqliteConnectionManager::file(db_url);
    Ok(Pool::new(manager)?)
}

fn schema_version(conn: &Connection) -> rusqlite::Result<i16> {
    conn.query_row("SELECT user_version FROM pragma_user_version", [], |row| {
        row.get(0)
    })
}
