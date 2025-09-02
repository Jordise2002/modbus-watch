use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};

use crate::common::model::{ModbusTable, Value, ValueFormattingParams};

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct ServedValue {
    pub id: String,
    pub starting_address: u16,
    pub table: ModbusTable,

    #[serde(flatten)]
    pub formatting_params: ValueFormattingParams,

    pub default_value: Value,
}

impl ServedValue {
    pub fn validate(&self, max_registers: u16) -> Result<()> {
        self.formatting_params.validate(self.table.clone())?;

        let register_size = self.table.register_size() as u16;

        let ending_bit =
            self.formatting_params.starting_bit as u16 + self.formatting_params.bit_length;

        let register_ammount = if ending_bit % register_size == 0 {
            ending_bit / register_size
        } else {
            ending_bit / register_size + 1
        };

        let ending_address = self.starting_address + register_ammount - 1;

        if ending_address > max_registers {
            return Err(anyhow!(
                "Value {} doesn't fit in tablel. Last register is {} and max grid register is {}",
                self.id,
                ending_address,
                max_registers
            ));
        }
        Ok(())
    }
}
