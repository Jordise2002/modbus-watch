use serde::{Serialize, Deserialize};
use anyhow::{Result, anyhow};
use std::{net::IpAddr, str::FromStr, time::Duration};

use crate::model::slave::PolledSlave;

fn default_port() -> u16
{
    502
}

fn default_ip() -> IpAddr
{
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
