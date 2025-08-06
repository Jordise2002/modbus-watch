use anyhow::Result;
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::params;
use serde::de::value;
use std::time::UNIX_EPOCH;
use tokio::sync::mpsc::Receiver;
use tracing::debug;
use tracing::error;
use tracing::instrument;

use crate::model::PolledConnection;
use crate::model::PolledValue;

mod tables;

pub struct InsertValueMessage {
    pub name: String,
    pub timestamp: std::time::SystemTime,
    pub value: Vec<u8>,
}
pub struct DbManager {
    path: std::path::PathBuf,
    db: Pool<SqliteConnectionManager>,
    insert_channel: Receiver<InsertValueMessage>,
}

impl DbManager {
    pub fn new(
        path: std::path::PathBuf,
        config: &Vec<PolledConnection>,
        insert_channel: Receiver<InsertValueMessage>,
    ) -> Result<Self> {
        let db = Self::build_db(path.clone())?;
        let db_manager = DbManager {
            db,
            insert_channel,
            path,
        };

        db_manager.init_db(config)?;

        Ok(db_manager)
    }

    pub async fn listen(&mut self) {
        debug!("DB {} started listening", self.path.to_string_lossy());
        loop {
            let insert = self.insert_channel.recv().await.unwrap();
            let result = self.insert_modbus_poll(insert.name.clone(), insert.value.clone(), insert.timestamp);
            if let Err(err) = result {
                error!("error inserting poll into db: {}", err.to_string());
            }

            debug!("Inserted poll {:?} for value {} into db", insert.value, insert.name);
        }
    }

    #[instrument]
    fn build_db(path: std::path::PathBuf) -> Result<Pool<SqliteConnectionManager>> {
        let db = SqliteConnectionManager::file(path);

        let db_pool = Pool::new(db)?;

        let conn = db_pool.get()?;

        conn.execute(tables::VALUE_TABLE, [])?;
        debug!("Built value table");

        conn.execute(tables::POLL_TABLE, [])?;
        debug!("Built poll table");

        conn.execute(tables::AGGREGATES_TABLE, [])?;
        debug!("Built aggregates table");

        Ok(db_pool)
    }

    fn init_db(&self, config: &Vec<PolledConnection>) -> Result<()> {
        for connection_config in config {
            for slave_config in &connection_config.slaves {
                for value_config in &slave_config.values {
                    self.insert_modbus_value(value_config, slave_config.id)?;
                }
            }
        }

        Ok(())
    }

    fn insert_modbus_value(&self, config: &PolledValue, slave_id: u8) -> Result<()> {
        let table_name = format!("{:?}", config.table);
        let config_json = serde_json::to_string(&config)?;
        let query = "INSERT OR REPLACE INTO modbus_values (
            name, address, modbus_table, slave_id, config
        ) VALUES (?, ?, ?, ?, ?)";

        let conn = self.db.get()?;
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
        &self,
        name: String,
        value: Vec<u8>,
        timestamp: std::time::SystemTime,
    ) -> Result<()> {
        let query = "INSERT INTO modbus_polls (value_id, timestamp, value) VALUES (?, ?, ?)";
        let conn = self.db.get()?;

        let secs_since_epoch = timestamp.duration_since(UNIX_EPOCH)?.as_secs();

        let _rows = conn.execute(&query, params![name, secs_since_epoch, value])?;

        Ok(())
    }
}
