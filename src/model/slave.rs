use serde::{Deserialize, Serialize};
use anyhow::{Result, anyhow};

use crate::model::value::PolledValue;


fn default_slave_id() -> u8 {
    1
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PolledSlave {
    #[serde(default)]
    pub config: PolledSlaveConfig,
    #[serde(default = "default_slave_id")]
    pub id: u8,
    pub addresses: Vec<PolledValue>,
}

impl PolledSlave {
    pub fn validate(&self) -> Result<()> {
        let mut error_string = String::new();

        for address in &self.addresses {
            if let Err(err) = address.validate() {
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
    pub allow_gaps_in_queries: bool,
}

impl Default for PolledSlaveConfig {
    fn default() -> Self {
        PolledSlaveConfig {
            max_register_ammount: 255,
            allow_gaps_in_queries: false,
        }
    }
}