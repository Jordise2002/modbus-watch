use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};

use crate::model::{DataType, Endianness, ModbusTable};

const MAX_VALUE_BIT_LENGTH: u16 = 64;

fn default_starting_bit() -> u8 {
    0
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PolledValue {
    pub id: String,
    pub starting_address: u16,

    #[serde(default = "default_starting_bit")]
    pub starting_bit: u8,
    pub bit_length: u16,

    pub data_type: DataType,
    pub table: ModbusTable,

    pub endianness: Endianness,
}

impl PolledValue {
    pub fn validate(&self) -> Result<()> {
        if self.data_type != DataType::Boolean
            && (self.table == ModbusTable::Coils || self.table == ModbusTable::DiscreteInput)
        {
            return Err(anyhow!(
                "Coils and DiscreteInput tables only support Boolean data types"
            ));
        }

        if (self.table == ModbusTable::Coils || self.table == ModbusTable::DiscreteInput)
            && self.starting_bit != 0
        {
            return Err(anyhow!("Coils and DiscreteInput tables have a maximum register size of 0, starting bit {} was provided!", self.starting_bit));
        }

        if (self.table == ModbusTable::Coils || self.table == ModbusTable::DiscreteInput)
            && self.bit_length != 1
        {
            return Err(anyhow!(
                "Coils and DiscreteInput tables have a maximum bit length of 1, {} was provided",
                self.bit_length
            ));
        }

        if self.bit_length < self.data_type.min_bit_size() {
            return Err(anyhow!(
                "Minimum byte size for {:?} is {}",
                self.data_type,
                self.data_type.min_bit_size()
            ));
        }

        if self.bit_length > MAX_VALUE_BIT_LENGTH {
            return Err(anyhow!(
                "Bit length ({}) is too high, maximum length is {}",
                self.bit_length,
                MAX_VALUE_BIT_LENGTH
            ));
        }

        Ok(())
    }
}
