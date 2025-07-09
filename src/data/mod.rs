use crate::model::PolledConnection;
use anyhow::{anyhow, Result};
use r2d2::{Pool, PooledConnection};
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::params;
use tokio::sync::mpsc::Receiver;
use std::path::PathBuf;

mod tables;

pub struct InsertValueMessage {
    name: String,
    timestamp: std::time::Instant,
    value: Vec<u8>
}
pub struct DbManager {
    db: Pool<SqliteConnectionManager>,
    insert_channel: Receiver<InsertValueMessage>
}

impl DbManager {
    pub fn new(path: PathBuf, config: &Vec<PolledConnection>, insert_channel: Receiver<InsertValueMessage>) -> Result<Self> {
        let db = Self::build_db(path)?;
        let db_manager = DbManager {db, insert_channel};

        db_manager.init_db(config)?;

        Ok(db_manager)
    }

    fn build_db(path: PathBuf) -> Result<Pool<SqliteConnectionManager>> {
        let db = SqliteConnectionManager::file(path);

        let db_pool = Pool::new(db)?;

        let conn = db_pool.get()?;

        conn.execute(tables::VALUE_TABLE, [])?;

        conn.execute(tables::POLL_TABLE, [])?;

        conn.execute(tables::AGGREGATES_TABLE, [])?;

        Ok(db_pool)
    }

    fn init_db(&self, config: &Vec<PolledConnection>) -> Result<()> {
        let conn = self.db.get()?;

        for connection_config in config {
            for slave_config in &connection_config.slaves {
                for value_config in &slave_config.values {
                    let query = tables::insert_modbus_value(value_config)?;
                    conn.execute(&query, params![])?;
                }
            }
        }

        Ok(())
    }
}
