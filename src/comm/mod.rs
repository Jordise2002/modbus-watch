use std::collections::HashMap;

use crate::{comm::context::ModbusCommContext, model::{ModbusTable, PolledConnection}};
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;

mod query;
mod context;

type AddressAndFunction = (ModbusTable, u16);
pub struct ModbusWatcher {
    db: Pool<SqliteConnectionManager>,
    contexts: Vec<ModbusCommContext>,
}

impl ModbusWatcher {
    pub fn new(config: Vec<PolledConnection>, db: Pool<SqliteConnectionManager>) -> Self {
        let mut contexts = vec![];
        
        for connection in config {
            contexts.push(ModbusCommContext::new(connection));
        }

        ModbusWatcher { db, contexts}
    }
}
