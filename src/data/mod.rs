use anyhow::Result;
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::params;
use sql_builder::SqlBuilder;
use std::path::PathBuf;
use std::time::UNIX_EPOCH;
use tokio::sync::mpsc::Receiver;

use crate::model::PolledConnection;
use crate::model::PolledValue;

mod tables;

pub struct InsertValueMessage {
    pub name: String,
    pub timestamp: std::time::SystemTime,
    pub value: Vec<u8>,
}
pub struct DbManager {
    db: Pool<SqliteConnectionManager>,
    insert_channel: Receiver<InsertValueMessage>,
}

impl DbManager {
    pub fn new(
        path: PathBuf,
        config: &Vec<PolledConnection>,
        insert_channel: Receiver<InsertValueMessage>,
    ) -> Result<Self> {
        let db = Self::build_db(path)?;
        let db_manager = DbManager { db, insert_channel };

        db_manager.init_db(config)?;

        Ok(db_manager)
    }

    pub async fn listen(&mut self) {
        loop {
            let insert = self.insert_channel.blocking_recv().unwrap();
            let _result= self.insert_modbus_poll(insert.name, insert.value, insert.timestamp);
        }
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
        for connection_config in config {
            for slave_config in &connection_config.slaves {
                for value_config in &slave_config.values {
                    self.insert_modbus_value(value_config)?;
                }
            }
        }

        Ok(())
    }

    fn insert_modbus_value(&self, config: &PolledValue) -> Result<()> {
        let table_name = format!("{:?}", config.table);
        let config_json = serde_json::to_string(&config)?;
        let query = SqlBuilder::insert_into("modbus_values")
            .fields(&["name", "address", "modbus_table", "slave_id", "config"])
            .values(&[
                &config.id,
                &config.starting_address.to_string(),
                &table_name,
                &config.id,
                &config_json,
            ])
            .sql()?;

        let conn = self.db.get()?;
        let _rows = conn.execute(&query, params![])?;
        Ok(())
    }


pub fn insert_modbus_poll(&self, name: String, value: Vec<u8>, timestamp: std::time::SystemTime) -> Result<()>
{
    let query = SqlBuilder::insert_into("modbus_polls").fields(&["value_id", "timestamp", "value"]).sql()?;

    let conn = self.db.get()?;

    let secs_since_epoch = timestamp.duration_since(UNIX_EPOCH)?.as_secs();

    let _rows = conn.execute(&query, params![name, secs_since_epoch, value])?;
    
    Ok(())
}

}
