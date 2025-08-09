use crate::model::PolledValue;

use anyhow::Result;
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::params;
use std::time::UNIX_EPOCH;

pub fn insert_modbus_value(
    conn: &r2d2::PooledConnection<SqliteConnectionManager>,
    config: &PolledValue,
    slave_id: u8,
) -> Result<()> {
    let table_name = format!("{:?}", config.table);
    let config_json = serde_json::to_string(&config)?;
    let query = "INSERT OR REPLACE INTO modbus_values (
            name, address, modbus_table, slave_id, config
        ) VALUES (?, ?, ?, ?, ?)";

    let _rows = conn.execute(
        &query,
        params![
            config.id,
            config.starting_address,
            table_name,
            slave_id,
            config_json
        ],
    )?;
    Ok(())
}

pub fn insert_modbus_poll(
    conn: &r2d2::PooledConnection<SqliteConnectionManager>,
    name: String,
    value: Vec<u8>,
    timestamp: std::time::SystemTime,
) -> Result<()> {
    let query = "INSERT INTO modbus_polls (value_id, timestamp, value) VALUES (?, ?, ?)";

    let secs_since_epoch = timestamp.duration_since(UNIX_EPOCH)?.as_secs();

    let _rows = conn.execute(&query, params![name, secs_since_epoch, value])?;

    Ok(())
}
