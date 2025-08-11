use std::time::UNIX_EPOCH;

use crate::data::ModbusPoll;
use crate::model::{DataType, Value};
use crate::value_processing;

use anyhow::Result;
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::params;

pub fn get_last_poll(
    conn: &r2d2::PooledConnection<SqliteConnectionManager>,
    value_id: String,
    data_type: DataType,
) -> Result<ModbusPoll> {
    let (secs_since_epoch, value_bytes): (i64, Vec<u8>) = conn.query_row(
        "SELECT timestamp, value
     FROM modbus_polls
     WHERE value_id = ?
     ORDER BY timestamp DESC
     LIMIT 1;",
        [value_id],
        |row| Ok((row.get(0)?, row.get(1)?)),
    )?;

    let value = value_processing::format_value(value_bytes, &data_type)?;

    Ok(ModbusPoll {
        value,
        secs_since_epoch,
    })
}

pub fn get_polls_between(
    conn: &r2d2::PooledConnection<SqliteConnectionManager>,
    value_id: &String,
    data_type: &DataType,
    start_time: std::time::SystemTime,
    finish_time: std::time::SystemTime,
) -> Result<Vec<Value>> {
    let start_time = start_time.duration_since(UNIX_EPOCH)?.as_secs();
    let finish_time = finish_time.duration_since(UNIX_EPOCH)?.as_secs();

    let mut stmt = conn.prepare(
        "SELECT value
         FROM modbus_polls
         WHERE value_id = ?
           AND timestamp BETWEEN ? AND ?",
    )?;

    let rows = stmt.query_map(params![value_id, start_time, finish_time], |row| {
        row.get::<_, Vec<u8>>(0)
    })?;

    let raw_values: Vec<Vec<u8>> = rows.filter_map(Result::ok).collect();

    let mut result = vec![];

    for raw_value in raw_values {
        result.push(value_processing::format_value(raw_value, data_type)?);
    }

    Ok(result)
}
