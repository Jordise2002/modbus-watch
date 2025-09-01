use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};

use crate::client::model::value::PolledValue;

fn default_slave_id() -> u8 {
    1
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PolledSlave {
    #[serde(default)]
    pub config: PolledSlaveConfig,
    #[serde(default = "default_slave_id")]
    pub id: u8,
    pub values: Vec<PolledValue>,
}

impl PolledSlave {
    pub fn validate(&self) -> Result<()> {
        let mut error_string = String::new();

        for address in &self.values {
            if let Err(err) = address.validate(self.config.max_register_ammount) {
                error_string += &format!("\t\t{}: {}\n", address.id, err);
            }
        }

        if error_string.is_empty() {
            Ok(())
        } else {
            Err(anyhow!(error_string))
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PolledSlaveConfig {
    pub max_register_ammount: u32,
    pub max_gap_size_in_query: u32,
}

impl Default for PolledSlaveConfig {
    fn default() -> Self {
        PolledSlaveConfig {
            max_register_ammount: 255,
            max_gap_size_in_query: 0,
        }
    }
}
