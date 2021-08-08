use crate::{
    conf::{Conf, Migration},
    provider::{Ecb, Iex},
};
use color_eyre::Report;
use figment::Figment;
use futures::join;
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
            migrate(&mut conn, version).unwrap_or_else(|e| {
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

pub fn migrate(conn: &mut Connection, target_version: DbVersion) -> Result<(), Report> {
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

async fn sync(args: &[String], rocket_conf: &Figment) -> Result<(), Report> {
    let conf = Conf::new()?;
    let mut ecb = Ecb::new(conf.providers.ecb, connect(&rocket_conf)?);
    let mut iex = Iex::new(conf.providers.iex, connect(&rocket_conf)?);

    match args.len() {
        0 => {
            join!(ecb.schedule(), iex.schedule());
        }
        1 => {
            if args.get(0).unwrap_or(&"".to_string()) == "now" {
                ecb.sync().await?;
                iex.sync().await?;
            } else {
                error!(?args, "Invalid arguments");
                exit(1);
            }
        }
        _ => error!(?args, "Invalid arguments"),
    }

    Ok(())
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
