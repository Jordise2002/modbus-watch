use tokio::sync::mpsc::Sender;

use crate::{comm::context::ModbusCommContext, data::InsertValueMessage, model::PolledConnection};

use anyhow::Result;

mod context;
mod value_processing;

pub struct ModbusWatcher {
    contexts: Vec<ModbusCommContext>,
}

impl ModbusWatcher {
    pub fn new(config: Vec<PolledConnection>, insert_channel: Sender<InsertValueMessage>) -> Self {
        let mut contexts = vec![];
        
        for connection in config {
            contexts.push(ModbusCommContext::new(connection, insert_channel.clone()));
        }

        ModbusWatcher { contexts}
    }

    pub async fn watch(& mut self) -> Result<()> {
        for context in & mut self.contexts
        {
            context.watch().await?;
        }

        Ok(())
    }
}
