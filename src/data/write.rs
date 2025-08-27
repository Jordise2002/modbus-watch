use crate::{
    aggregations::{AggregationInfo, Period},
    model::PolledValue,
    value_processing,
};

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

pub fn insert_modbus_aggregate(
    conn: &r2d2::PooledConnection<SqliteConnectionManager>,
    aggregate_info: AggregationInfo,
) -> Result<()> {
    let period = aggregate_info.period as u8;

    let start = aggregate_info
        .start_time
        .duration_since(UNIX_EPOCH)?
        .as_secs();
    let finish = aggregate_info
        .finish_time
        .duration_since(UNIX_EPOCH)?
        .as_secs();

    let average = value_processing::value_to_bytes(aggregate_info.aggregation.average);
    let median = value_processing::value_to_bytes(aggregate_info.aggregation.median);
    let moda = value_processing::value_to_bytes(aggregate_info.aggregation.moda);

    let min = value_processing::value_to_bytes(aggregate_info.aggregation.min);
    let max = value_processing::value_to_bytes(aggregate_info.aggregation.max);

    let query = "INSERT INTO modbus_aggregates 
    (value_id, period, start, finish, average, median, min, max, moda, ammount)
    VALUES(?, ?, ?, ?, ?, ?, ?, ?, ?, ?)";

    let _rows = conn.execute(
        &query,
        params![
            aggregate_info.value_id,
            period,
            start,
            finish,
            average,
            median,
            min,
            max,
            moda,
            aggregate_info.aggregation.ammount
        ],
    )?;

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
