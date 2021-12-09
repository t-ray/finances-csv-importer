use log::{info, debug, error};
use sqlx::{self, Database, Executor};

use sqlx::pool::Pool;
use sqlx::postgres::{PgPoolOptions, PgConnectOptions, Postgres};

use crate::config;
use crate::domain::CsvRecord;

type PgPool = sqlx::pool::Pool<sqlx::postgres::Postgres>;
type PgTx<'a> = sqlx::Transaction<'a, Postgres>;

#[derive(Debug)]
pub enum DatabaseError {
    ConnectionError,
}

impl DatabaseError {
    fn connection() -> Self {
        DatabaseError::ConnectionError
    }
}

impl std::fmt::Display for DatabaseError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match &self {
            Self::ConnectionError => write!(f, "Could not connect to database."),
        }
    }
}

impl std::error::Error for DatabaseError {
    fn description(&self) -> &str {
        match self {
            Self::ConnectionError => "Could not connect to database."
        }
    }
}

pub async fn connect(c: &config::DatabaseConfig) -> Result<Pool<Postgres>, DatabaseError> {

    let connect_options = PgConnectOptions::from(c);

    info!("Attempting to connect to database.");
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect_with(connect_options).await
        .map_err(|_| DatabaseError::connection())?;

    // Make a simple query to return the given parameter (use a question mark `?` instead of `$1` for MySQL)
    let _: (i64,) = sqlx::query_as("SELECT $1")
        .bind(150_i64)
        .fetch_one(&pool).await
        .map_err(|_| DatabaseError::connection())?;

    info!("Successfully connected to database.");

    Ok(pool)
}

pub async fn import(records: &[CsvRecord], table_name: &str, pool: &PgPool) -> Result<(), sqlx::Error> {

    let chunk_size = 50;

    for chunk in records.chunks(chunk_size) {
        debug!("Attempting to insert chunk of {} records.", chunk.len());
        let mut tx = pool.begin().await?;

        for row in records {
            if insert_single_row(row, table_name, &mut tx).await.is_err() {
                error!("Could not insert row {}/{}", row.account, row.id);
            }
        }
        
        tx.commit().await?;
        debug!("Chunk of {} records inserted and committed.", chunk.len())
    }

    
    Ok(())
}

async fn insert_single_row(row: &CsvRecord, table_name: &str, tx: &mut PgTx<'_>) -> Result<(), sqlx::Error> {
    let sql = format!("INSERT INTO {table_name}(account, tx_id, tx_date, amount, balance, vendor, digits, transaction_type, category, subcategory, notes) 
        VALUES($1, $2, $3, $4::numeric, $5::numeric, $6, $7, $8, $9, $10, $11) ON CONFLICT DO NOTHING", 
        table_name = table_name);

    let insert_result = sqlx::query(&sql)
        .bind(&row.account)
        .bind(row.id as i32)
        .bind(&row.date)
        .bind(&row.amount.to_string())
        .bind(&row.balance.to_string())
        .bind(&row.vendor)
        .bind(&row.digits)
        .bind(&row.transaction_type)
        .bind(&row.category)
        .bind(&row.subcategory)
        .bind(&row.notes)
        .execute(tx).await;

    if insert_result.is_err() {
        error!("Could not insert row: {}, {}", row.account, row.id);
    }

    Ok(())
}

