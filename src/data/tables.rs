use std::fmt;

use anyhow::Result;
use sql_builder::SqlBuilder;

use crate::model::PolledValue;

pub const VALUE_TABLE: &str = "CREATE TABLE IF NOT EXISTS modbus_values (
                                name TEXT PRIMARY KEY,
                                address INTEGER NOT NULL,
                                modbus_table TEXT NOT NULL,
                                slave_id INTEGER NOT NULL,
                                config TEXT
                            );";

pub const POLL_TABLE: &str = "CREATE TABLE IF NOT EXISTS modbus_polls (
                                id INTEGER PRIMARY KEY AUTOINCREMENT,
                                value_id TEXT NOT NULL REFERENCES modbus_values(id),
                                timestamp DATETIME NOT NULL,
                                value blob
                            );";

pub const AGGREGATES_TABLE: &str = "CREATE TABLE IF NOT EXISTS modbus_aggregates (
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

pub fn insert_modbus_value(config: &PolledValue) -> Result<String> {
    let table_name = format!("{:?}", config.table);
    let config_json = serde_json::to_string(&config)?;
    let insert = SqlBuilder::insert_into("modbus_values")
        .fields(&["name", "address", "modbus_table", "slave_id", "config"])
        .values(&[
            &config.id,
            &config.starting_address.to_string(),
            &table_name,
            &config.id,
            &config_json,
        ])
        .sql();
    insert
}
