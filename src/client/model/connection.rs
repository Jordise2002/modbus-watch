use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::{collections::HashSet, net::IpAddr, str::FromStr, time::Duration};

use crate::client::model::slave::PolledSlave;

fn default_port() -> u16 {
    502
}

fn default_ip() -> IpAddr {
    IpAddr::from_str("127.0.0.1").unwrap()
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PolledConnection {
    #[serde(default)]
    pub config: PolledConnectionConfig,
    #[serde(default = "default_ip")]
    pub ip: IpAddr,
    #[serde(default = "default_port")]
    pub port: u16,
    pub slaves: Vec<PolledSlave>,
}

impl PolledConnection {
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

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PolledConnectionConfig {
    pub max_simultaneous_connections: u32,
    #[serde(with = "humantime_serde")]
    pub max_response_time: Duration,
}

impl Default for PolledConnectionConfig {
    fn default() -> Self {
        PolledConnectionConfig {
            max_simultaneous_connections: 1,
            max_response_time: Duration::from_secs(1),
        }
    }
}
