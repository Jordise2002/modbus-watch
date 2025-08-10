use crate::{aggregations::Period, model::PolledValue};

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

pub fn delete_exceeding_polls(
    conn: &r2d2::PooledConnection<SqliteConnectionManager>,
    name: String,
    max_polls: u64,
) -> Result<()> {
    conn.execute(
        "
    DELETE FROM modbus_polls
    WHERE id IN (
        SELECT id FROM modbus_polls
        WHERE value_id = ?
        ORDER BY timestamp DESC
        LIMIT -1 OFFSET ?
    )",
        [name, max_polls.to_string()],
    )?;

    Ok(())
}

pub fn delete_exceeding_aggregations(
    conn: &r2d2::PooledConnection<SqliteConnectionManager>,
    name: String,
    period: Period,
    max_aggregations: u64,
) -> Result<()> {
    let period = format!("{:?}", period);
    conn.execute(
        "
        DELETE FROM modbus_aggregates
        WHERE id IN (
            SELECT id FROM modbus_aggregates
            WHERE value_id = ?
            AND period = ?
            ORDER BY start DESC
            LIMIT -1 OFFSET ?
        )",
        [name, period, max_aggregations.to_string()],
    )?;

    Ok(())
}
