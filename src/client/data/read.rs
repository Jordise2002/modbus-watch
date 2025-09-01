use std::time::UNIX_EPOCH;

use crate::client::aggregations::{Aggregation, AggregationInfo, Period};
use crate::client::data::ModbusPoll;
use crate::common::model::DataType;
use crate::common::value_processing;

use anyhow::Result;
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::params;

pub fn get_last_poll(
    conn: &r2d2::PooledConnection<SqliteConnectionManager>,
    value_id: String,
    data_type: DataType,
) -> Result<ModbusPoll> {
    let (secs_since_epoch, value_bytes): (u64, Vec<u8>) = conn.query_row(
        "SELECT timestamp, value
     FROM modbus_polls
     WHERE value_id = ?
     ORDER BY timestamp DESC
     LIMIT 1;",
        [value_id.clone()],
        |row| Ok((row.get(0)?, row.get(1)?)),
    )?;

    let value = value_processing::format_value(value_bytes, &data_type)?;

    Ok(ModbusPoll {
        value_id,
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
) -> Result<Vec<ModbusPoll>> {
    let start_time = start_time.duration_since(UNIX_EPOCH)?.as_secs();
    let finish_time = finish_time.duration_since(UNIX_EPOCH)?.as_secs();

    let mut stmt = conn.prepare(
        "SELECT value, timestamp
         FROM modbus_polls
         WHERE value_id = ?
           AND timestamp BETWEEN ? AND ?",
    )?;

    let mut rows = stmt.query(params![value_id.clone(), start_time, finish_time])?;
    
    let mut result = vec![];

    while let Some(row) = rows.next()? {
        let value: Vec<u8> = row.get(0)?;
        let timestamp: u64 = row.get(1)?;

        let value = value_processing::format_value(value, data_type)?;

        let poll = ModbusPoll {
            value_id: value_id.clone(),
            value,
            secs_since_epoch: timestamp
        };

        result.push(poll);
    }

    Ok(result)
}

pub fn get_aggregates_between(
    conn: &r2d2::PooledConnection<SqliteConnectionManager>,
    value_id: &String,
    data_type: &DataType,
    start_time: std::time::SystemTime,
    finish_time: std::time::SystemTime,
    max_period: Period,
    min_period: Period,
) -> Result<Vec<AggregationInfo>> {
    let start_time = start_time.duration_since(UNIX_EPOCH)?.as_secs();
    let finish_time = finish_time.duration_since(UNIX_EPOCH)?.as_secs();

    let min_period = min_period as u8;
    let max_period = max_period as u8;

    let mut stmt = conn.prepare(
        "SELECT value_id, period, start, finish, average, median, moda, min, max, ammount
         FROM modbus_aggregates
         WHERE start >= ?1
           AND finish <= ?2
           AND period BETWEEN ?3 AND ?4
           AND value_id == ?5",
    )?;

    let mut rows = stmt.query(params![start_time, finish_time, min_period, max_period, value_id])?;

    let mut result = vec![];

    while let Some(row) = rows.next()? {
        let value_id: String = row.get(0)?;
        let period: u8 = row.get(1)?;
        let start_time: u64 = row.get(2)?;
        let finish_time: u64 = row.get(3)?;
        let average: Vec<u8> = row.get(4)?; // BLOB
        let median: Vec<u8> = row.get(5)?; // BLOB
        let moda: Vec<u8> = row.get(6)?; // BLOB
        let min: Vec<u8> = row.get(7)?; // BLOB
        let max: Vec<u8> = row.get(8)?; // BLOB
        let ammount: u64 = row.get(9)?;

        let start_time = UNIX_EPOCH + std::time::Duration::from_secs(start_time);
        let finish_time = UNIX_EPOCH + std::time::Duration::from_secs(finish_time);

        let period = Period::from_repr(period)?;
        let average = value_processing::format_value(average, data_type)?;
        let median = value_processing::format_value(median, data_type)?;
        let moda = value_processing::format_value(moda, data_type)?;
        let min = value_processing::format_value(min, data_type)?;
        let max = value_processing::format_value(max, data_type)?;

        let aggregation = Aggregation {
            average, median, moda, min, max, ammount
        };

        let aggregation = AggregationInfo {
            value_id,
            period,
            start_time,
            finish_time,
            aggregation
        };

        result.push(aggregation);
    }

    Ok(result)
}
