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
use std::{fs::remove_file, path::Path};
use tracing::{info, warn};

#[derive(Debug)]
pub enum DbVersion {
    Specific(i16),
    Latest,
}

pub async fn cli(args: &[String]) -> Result<()> {
    let first_arg = args.first().ok_or(Error::msg("No args provided"))?;

    match first_arg.as_str() {
        "drop" => cli_drop().context("Unable to drop database"),
        "migrate" => cli_migrate(args.get(1)),
        "sync" => cli_sync(&args[1..]).await,
        _ => Err(Error::msg("Unknown argument")),
    }
}

fn cli_drop() -> Result<()> {
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

fn cli_migrate(ver: Option<&String>) -> Result<()> {
    let ver = match ver {
        Some(ver) => DbVersion::Specific(ver.parse().context("Can't parse database version")?),
        None => DbVersion::Latest,
    };
    let mut conn = new_connection()?;
    migrate(&mut conn, ver)
}

pub fn migrate_to_latest(conn: &mut Connection) -> Result<()> {
    migrate(conn, DbVersion::Latest)
}

fn migrate(conn: &mut Connection, target_version: DbVersion) -> Result<()> {
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

async fn cli_sync(args: &[String]) -> Result<()> {
    let conf = Conf::new()?;
    let pool = new_pool()?;

    let ecb = Ecb::new(conf.providers.ecb, ExchangeRateRepository::new(&pool));
    let iex = Iex::new(conf.providers.iex, ExchangeRateRepository::new(&pool));

    match args.len() {
        0 => {
            let results = join_all(vec![ecb.schedule(), iex.schedule()]).await;

            for result in results {
                result?;
            }
        }
        1 => match args.first().unwrap().as_str() {
            "now" => {
                let results = join_all(vec![ecb.sync(), iex.sync()]).await;

                for result in results {
                    result?;
                }
            }
            _ => return Err(Error::msg("Unknown arguments")),
        },
        _ => return Err(Error::msg("Unknown arguments")),
    }

    Ok(())
}

fn new_connection() -> Result<Connection> {
    let db_url = Conf::new()?.db_url;
    Ok(Connection::open(db_url)?)
}

fn new_pool() -> Result<Pool<SqliteConnectionManager>> {
    let db_url = Conf::new()?.db_url;
    let manager = SqliteConnectionManager::file(db_url);
    Ok(Pool::new(manager)?)
}

fn schema_version(conn: &Connection) -> rusqlite::Result<i16> {
    conn.query_row("SELECT user_version FROM pragma_user_version", [], |row| {
        row.get(0)
    })
}
