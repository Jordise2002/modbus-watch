use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};

use crate::common::model::{ModbusTable, ValueFormattingParams};

fn default_max_polls_to_keep() -> Option<u64> {
    //Aprox three days of a 100ms poll time value
    Some(24 * 3 * 60 * 60 * 10)
}

fn default_max_minute_aggregations_to_keep() -> Option<u64> {
    //Three weeks of minutes aggregations
    Some(24 * 60 * 60 * 3 * 7)
}

fn default_max_hour_aggregations_to_keep() -> Option<u64> {
    //About a year
    Some(24 * 365)
}

fn default_max_day_aggregations_to_keep() -> Option<u64> {
    None
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PolledValue {
    pub id: String,
    pub starting_address: u16,
    pub table: ModbusTable,

    #[serde(flatten)]
    pub formatting_params: ValueFormattingParams,
    #[serde(with = "humantime_serde")]
    pub poll_time: std::time::Duration,

    #[serde(default = "default_max_polls_to_keep")]
    pub max_polls_to_keep: Option<u64>,
    #[serde(default = "default_max_minute_aggregations_to_keep")]
    pub max_minute_aggregations_to_keep: Option<u64>,
    #[serde(default = "default_max_hour_aggregations_to_keep")]
    pub max_hour_aggregations_to_keep: Option<u64>,
    #[serde(default = "default_max_day_aggregations_to_keep")]
    pub max_day_aggregations_to_keep: Option<u64>,
}

impl PolledValue {
    pub fn validate(&self, max_register_ammount: u32) -> Result<()> {

        self.formatting_params.validate(self.table.clone())?;

        let register_size = self.table.register_size() as u16;

        let ending_bit =
            self.formatting_params.starting_bit as u16 + self.formatting_params.bit_length;

        let register_ammount = if ending_bit % register_size == 0 {
            ending_bit / register_size
        } else {
            ending_bit / register_size + 1
        };

        if register_ammount as u32 > max_register_ammount {
            return Err(anyhow!(
                "Max register ammount for query is {}, this query would contain {} registers",
                max_register_ammount,
                register_ammount
            ));
        }

        Ok(())
    }
}
