use std::path::PathBuf;
use r2d2::{Pool, PooledConnection};
use r2d2_sqlite::SqliteConnectionManager;
use anyhow::{Result, anyhow};
use sql_builder::SqlBuilder;

const VALUE_TABLE: &str = "CREATE TABLE IF NOT EXISTS modbus_values (
                                id INTEGER PRIMARY KEY AUTOINCREMENT,
                                address INTEGER NOT NULL,
                                modbus_table TEXT NOT NULL,
                                slave_id INTEGER NOT NULL,
                                config TEXT
                            );";

const POLL_TABLE: &str = "CREATE TABLE IF NOT EXISTS modbus_polls (
                                id INTEGER PRIMARY KEY AUTOINCREMENT,
                                value_id INTEGER NOT NULL REFERENCES modbus_values(id),
                                timestamp DATETIME NOT NULL,
                                value blob
                            );";
                            
const AGGREGATES_TABLE: &str = "CREATE TABLE IF NOT EXISTS modbus_aggregates (
                                    id INTEGER PRIMARY KEY,
                                    value_id INTEGER NOT NULL REFERENCES modbus_values(id),
                                    period TEXT NOT NULL,
                                    start DATETIME NOT NULL,
                                    finish DATETIME NOT NULL,
                                    average blob,
                                    median blob,
                                    min blob,
                                    max blob,
                                    ammount INTEGER 
                                );";

pub async fn build_db(conn: PooledConnection<SqliteConnectionManager>) -> Result<()>
{
    conn.execute(VALUE_TABLE, [])?;

    conn.execute(POLL_TABLE, [])?;

    conn.execute(AGGREGATES_TABLE, [])?;

    Ok(())
}

pub async fn init_db(path: PathBuf) -> Result<Pool<SqliteConnectionManager>> 
{
    let db = SqliteConnectionManager::file(path);

    let db_pool = Pool::new(db)?;

    let conn = db_pool.get()?;

    build_db(conn).await?;

    Ok(db_pool)
}