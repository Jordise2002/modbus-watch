use std::u16;
use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};

use crate::{common::model::ModbusTable, server::model::value::ServedValue};

fn default_id() -> u8 {
    1
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ServedSlave {
    #[serde(default = "default_id")]
    pub id: u8,

    #[serde(flatten)]
    pub config: ServedSlaveConfig,
    pub values: Vec<ServedValue>
}

impl ServedSlave {
    pub fn validate(&self) -> Result<()> {
        let mut error_string = String::new();
        
        for value in &self.values {
            let max_registers = match value.table {
                ModbusTable::Coils => {
                    self.config.max_coils
                }
                ModbusTable::DiscreteInput => {
                    self.config.max_discrete_inputs
                }
                ModbusTable::HoldingRegisters => {
                    self.config.max_holding_registers
                }
                ModbusTable::InputRegisters =>  {
                    self.config.max_input_registers
                }
            };
            if let Err(error) = value.validate(max_registers)
            {
                error_string += &error.to_string();
            }
        }

        if error_string.is_empty() {
            Ok(())
        }
        else {
            Err(anyhow!(error_string))
        }
    }
}

fn default_response_delay() -> Option<std::time::Duration> {
    None
}

fn default_grid_size() -> u16 {
    u16::MAX
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ServedSlaveConfig {
    #[serde(with = "humantime_serde", default = "default_response_delay")]
    response_delay: Option<std::time::Duration>,

    #[serde(default = "default_grid_size")]
    max_coils: u16,
    #[serde(default = "default_grid_size")]
    max_discrete_inputs: u16,
    #[serde(default = "default_grid_size")]
    max_holding_registers: u16,
    #[serde(default = "default_grid_size")]
    max_input_registers: u16,
}
