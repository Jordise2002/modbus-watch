use crate::{comm::context::ModbusCommContext, model::{ModbusTable, PolledConnection}};

use anyhow::Result;
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;

mod context;

pub struct ModbusWatcher {
    contexts: Vec<ModbusCommContext>,
}

impl ModbusWatcher {
    pub fn new(config: Vec<PolledConnection>) -> Self {
        let mut contexts = vec![];
        
        for connection in config {
            contexts.push(ModbusCommContext::new(connection));
        }

        ModbusWatcher { contexts}
    }

    pub fn watch(& mut self) -> Result<()> {
        
        
        Ok(())
    }
}
