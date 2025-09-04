use std::sync::Arc;

use context::ModbusSlaveCommContext;
use crate::server::{model::ServedConnection, state::AppState};

mod context;
pub struct ModbusServer {
    pub contexts: Vec<Arc<ModbusSlaveCommContext>>,
}

impl ModbusServer {
    pub fn new(config: &Vec<ServedConnection>, app_state: AppState) -> Self {
        let mut contexts = vec![];
        for connection in config {
            let context = ModbusSlaveCommContext::new(connection, app_state.clone());
            contexts.push(context);
        }

        ModbusServer { contexts }
    }

    pub fn serve(&self) {
        for context in &self.contexts {
            ModbusSlaveCommContext::serve(context.clone());
        }
    }
}
