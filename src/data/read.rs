use crate::data::ModbusPoll;
use crate::model::DataType;
use crate::value_processing;

use anyhow::Result;
use r2d2_sqlite::SqliteConnectionManager;

pub fn get_last_poll(
    conn: &r2d2::PooledConnection<SqliteConnectionManager>,
    value_id: String,
    data_type: DataType
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

    Ok(ModbusPoll { value, secs_since_epoch })
}
