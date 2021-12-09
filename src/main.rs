mod config;
mod currency;
mod domain;
mod db;

use std::{error::Error, path::PathBuf};
use std::fmt;
use std::str::FromStr;

use csv::{ReaderBuilder};
use log::{debug, info, error};
use serde::{Deserialize, Deserializer};
use tokio;

use chrono::prelude::*;

use domain::CsvRecord;
use config::DatabaseConfig;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
type PgPool = sqlx::pool::Pool<sqlx::postgres::Postgres>;

#[tokio::main]
async fn main() -> Result<()> {

    env_logger::init();


    // let s = "9/22/2021 00:00:00 +00:00";
    // let d = DateTime::parse_from_str(s, "%m/%d/%Y %H:%M:%S %z")
    //     .map(|dt| dt.date());
    
    // println!("{:?}", d);

    let config = match config::parse_args() {
        Ok(c) => c,
        Err(e) => return Err(e)
    };

    let pool = match db::connect(&config.database).await {
        Ok(p) => p,
        Err(e) => return Err(Box::new(e))
    };
    
    match config.source {
        config::Source::File(f) => {
            import_file(&f, &config.database, &pool).await?;
        },
        config::Source::Directory(dir) => {
            import_directory(&dir, &config.database, &pool).await?;
        }
    }
    
    Ok(())
}

async fn import_directory(f: &PathBuf, db_config: &DatabaseConfig, pool: &PgPool) -> Result<()> {
    let paths = std::fs::read_dir(f)?;

    for entry in paths {
        if let Ok(f) = entry {
            let path = f.path();
            let _ = import_file(&path, db_config, pool).await;
        }
    }

    Ok(())
}

async fn import_file(f: &PathBuf, db_config: &DatabaseConfig, pool: &PgPool) -> Result<()> {

    return match read_file(f) {
        Ok(records) => {
            load_rows(&records, db_config, pool).await?;
            Ok(())
        },
        Err(e) => {
            error!("Could not read csv file. Aborting");
            Err(e)
        }
    }
}

async fn load_rows(rows: &[CsvRecord], db_config: &DatabaseConfig, pool: &PgPool) -> Result<()> {
    db::import(rows, &db_config.get_table_name(), pool).await?;
    Ok(())
}

fn read_file(f: &PathBuf) -> Result<Vec<CsvRecord>> {


    let abs_path = f.canonicalize()?;
    info!("Reading csv records from file {:?}", abs_path);

    let mut records = Vec::new();
    let mut reader = ReaderBuilder::new()
        .trim(csv::Trim::Headers)
        .from_path(f)?;

    let mut bad_rows = 0;

    for result in reader.deserialize::<CsvRecord>() {
        // let record: CsvRecord = result?;
        if let Ok(record) = result {
            records.push(record);
        } else {
            bad_rows += 1;
        }
    }

    info!("Read {} records from file. {} rows ignored because they could not be loaded.", records.len(), bad_rows);
    Ok(records)
}