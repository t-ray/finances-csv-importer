mod config;
mod currency;
mod db;
mod domain;

use std::path::Path;

use csv::ReaderBuilder;
use itertools::Itertools;
use log::{error, info};

use crate::domain::LoadOptions;
use config::DatabaseConfig;
use domain::CsvRecord;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
type PgPool = sqlx::pool::Pool<sqlx::postgres::Postgres>;

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();

    let config = match config::parse_args() {
        Ok(c) => c,
        Err(e) => return Err(e),
    };

    let pool = match db::connect(&config.database).await {
        Ok(p) => p,
        Err(e) => return Err(Box::new(e)),
    };

    if config.database.is_init() {
        if let Err(e) = db::init(&config.database, &pool).await {
            return Err(Box::new(e));
        }
    }

    let options = config.load_options;

    match config.source {
        config::Source::File(f) => {
            import_file(&f, options, &config.database, &pool).await?;
        }
        config::Source::Directory(dir) => {
            import_directory(&dir, options, &config.database, &pool).await?;
        }
    }

    Ok(())
}

async fn import_directory(
    f: &Path,
    options: LoadOptions,
    db_config: &DatabaseConfig,
    pool: &PgPool,
) -> Result<()> {
    let paths = std::fs::read_dir(f)?;

    for entry in paths.flatten() {
        let path = entry.path();
        let _ = import_file(&path, options, db_config, pool).await;
    }

    Ok(())
}

async fn import_file(
    f: &Path,
    options: LoadOptions,
    db_config: &DatabaseConfig,
    pool: &PgPool,
) -> Result<()> {
    return match read_file(f) {
        Ok(records) => {
            load_rows(&records, options, db_config, pool).await?;
            Ok(())
        }
        Err(e) => {
            error!("Could not read csv file. Aborting");
            Err(e)
        }
    };
}

async fn load_rows(
    rows: &[CsvRecord],
    options: LoadOptions,
    db_config: &DatabaseConfig,
    pool: &PgPool,
) -> Result<()> {
    let table_name = db_config.get_table_name();

    match options {
        LoadOptions::All => db::import(rows, &table_name, pool).await?,
        LoadOptions::New => load_new_rows(rows, &table_name, pool).await?,
    }

    Ok(())
}

async fn load_new_rows(rows: &[CsvRecord], table_name: &str, pool: &PgPool) -> Result<()> {
    // group by account
    for (account, group) in &rows.iter().group_by(|r| r.account.clone()) {
        let account_rows = group.collect::<Vec<_>>();
        if let Ok(max) = db::select_max_tx_for_account(&account, table_name, pool).await {
            let to_import = account_rows
                .iter()
                .filter(|r| r.id > max as u64)
                .cloned()
                .collect::<Vec<_>>();

            if !to_import.is_empty() {
                info!(
                    "Resuming import for account {} after tx {}. Attempting to import {} new rows.",
                    account,
                    max,
                    to_import.len()
                );
                let _ = db::import_refs(&to_import, table_name, pool).await?;
            }
        }
    }

    Ok(())
}

fn read_file(f: &Path) -> Result<Vec<CsvRecord>> {
    let abs_path = f.canonicalize()?;
    info!("Reading csv records from file {:?}", abs_path);

    let mut records = Vec::new();
    let mut reader = ReaderBuilder::new().trim(csv::Trim::Headers).from_path(f)?;

    let mut bad_rows = 0;

    for result in reader.deserialize::<CsvRecord>() {
        match result {
            Ok(record) => records.push(record),
            Err(e) => {
                error!("Skipping row Could not read row: {}", e);
                bad_rows += 1;
            }
        }
    }

    info!(
        "Read {} records from file. {} rows ignored because they could not be loaded.",
        records.len(),
        bad_rows
    );
    Ok(records)
}
