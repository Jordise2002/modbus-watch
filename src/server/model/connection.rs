use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

use crate::server::model::slave::ServedSlave;

fn default_port() -> u16 {
    502
}

#[derive(Serialize, Deserialize, Clone, PartialEq)]
pub struct ServedConnection {
    #[serde(default = "default_port")]
    pub port: u16,

    #[serde(flatten)]
    pub config: ServedConnectionConfig,
    pub slaves: Vec<ServedSlave>
}

impl ServedConnection {
    pub fn validate(&self) -> Result<()> {
        let mut error_string = String::new();

        for slave in &self.slaves {
            if let Err(err) = slave.validate() {
                error_string += &format!("\t{}:\n{}\n", slave.id, err);
            }
        }

        let mut name_set = HashSet::new();
        let mut repeated_set = HashSet::new();

        for slave in &self.slaves {
            for value in &slave.values {
                if !name_set.insert(value.id.clone()) && !repeated_set.contains(&value.id) {
                    error_string += &format!(
                        "Repeated value names: {} was defined more than once\n",
                        value.id
                    );
                    repeated_set.insert(value.id.clone());
                }
            }
        }

        if error_string.is_empty() {
            Ok(())
        } else {
            Err(anyhow!(error_string))
        }
    }
}

fn default_connection_time_to_live() -> std::time::Duration {
    std::time::Duration::from_secs(3)
}

#[derive(Serialize, Deserialize, Clone, PartialEq)]
pub struct ServedConnectionConfig {
    #[serde(default = "default_connection_time_to_live")]
    pub connection_time_to_live: std::time::Duration,
}
