use anyhow::Result;
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use std::sync::Arc;
use tokio::sync::mpsc::Receiver;
use tracing::debug;
use tracing::error;
use tracing::instrument;
use serde::{Serialize, Deserialize};

use crate::model::PolledConnection;

pub mod read;
mod write;
mod tables;

pub struct InsertValueMessage {
    pub name: String,
    pub timestamp: std::time::SystemTime,
    pub value: Vec<u8>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ModbusPoll {
    value: Vec<u8>,
    secs_since_epoch: i64
}

pub struct DbManager {
    path: std::path::PathBuf,
    db: Arc<Pool<SqliteConnectionManager>>,
    insert_channel: Receiver<InsertValueMessage>,
}

impl DbManager {
    pub fn new(
        path: std::path::PathBuf,
        config: &Vec<PolledConnection>,
        insert_channel: Receiver<InsertValueMessage>,
    ) -> Result<Self> {
        let db = Arc::new(Self::build_db(path.clone())?);
        let db_manager = DbManager {
            db,
            insert_channel,
            path,
        };

        db_manager.init_db(config)?;

        Ok(db_manager)
    }

    pub fn get_db(&self) -> Arc<Pool<SqliteConnectionManager>> {
        self.db.clone()
    }

    pub async fn listen(&mut self) {
        debug!("DB {} started listening", self.path.to_string_lossy());
        loop {
            let insert = self.insert_channel.recv().await.unwrap();
            let conn = self.db.get().unwrap();
            let result = write::insert_modbus_poll(
                &conn,
                insert.name.clone(),
                insert.value.clone(),
                insert.timestamp,
            );
            if let Err(err) = result {
                error!("error inserting poll into db: {}", err.to_string());
            }

            debug!(
                "Inserted poll {:?} for value {} into db",
                insert.value, insert.name
            );
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
        let conn = self.db.get()?;
        for connection_config in config {
            for slave_config in &connection_config.slaves {
                for value_config in &slave_config.values {
                    write::insert_modbus_value(&conn, value_config, slave_config.id)?;
                }
            }
        }

        Ok(())
    }
}
