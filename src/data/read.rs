use std::time::{Duration, UNIX_EPOCH};

use crate::data::ModbusPoll;

use anyhow::Result;
use r2d2_sqlite::SqliteConnectionManager;

pub fn get_last_poll(
    conn: &r2d2::PooledConnection<SqliteConnectionManager>,
    value_id: String,
) -> Result<ModbusPoll> {
    let (secs_since_epoch, value): (i64, Vec<u8>) = conn.query_row(
        "SELECT timestamp, value
     FROM modbus_polls
     WHERE value_id = ?
     ORDER BY timestamp DESC
     LIMIT 1;",
        [value_id],
        |row| Ok((row.get(0)?, row.get(1)?)),
    )?;

    Ok(ModbusPoll { value, secs_since_epoch })
}
