use std::collections::HashMap;
use anyhow::Result;
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use std::net::SocketAddr;
use tweakable_modbus::{ModbusAddress, ModbusTable, ModbusMasterConnection};

use crate::model::{PolledConnection, PolledValue};

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Query {
    pub starting_address: u16,
    pub ending_address: u16,
    pub ammount: u16,
    pub table: ModbusTable,
    pub slave_id: u8,
    pub poll_time: std::time::Duration,
}

pub struct ValueBinding {
    starting_address: ModbusAddress,
    needed_addresses: Vec<u16>,
    config: PolledValue,
}
pub struct ModbusCommContext {
    queries: Vec<Query>,
    value_bindings: Vec<ValueBinding>,
    config: PolledConnection,
}

impl ModbusCommContext {
    pub fn new(config: PolledConnection) -> Self {
        let queries = Self::build_queries(&config);

        let value_bindings = Self::build_value_bindings(&config);

        ModbusCommContext {
            config,
            queries,
            value_bindings,
        }
    }

    pub fn watch(&mut self, db: Pool<SqliteConnectionManager>) -> Result<()> {
        let mut queries_ordered_by_poll_time: HashMap<std::time::Duration, Vec<Query>> = HashMap::new();

        for query in &self.queries {
            queries_ordered_by_poll_time.entry(query.poll_time).or_default().push(query.clone());
        }

        let socket = SocketAddr::new(self.config.ip, self.config.port);
        let mut master_connection = ModbusMasterConnection::new_tcp(socket);


        Ok(())
    }

    fn build_value_bindings(config: &PolledConnection) -> Vec<ValueBinding> {
        let mut value_bindings = vec![];

        for slave in &config.slaves {
            for address in &slave.values {
                let starting_address = ModbusAddress {
                    address: address.starting_address,
                    table: address.table.to_tweakable_modbus_table(),
                    slave_id: slave.id,
                };

                let ending_bit = address.starting_bit as u16 + address.bit_length;

                let register_size = if address.table == crate::model::ModbusTable::Coils
                    || address.table == crate::model::ModbusTable::DiscreteInput
                {
                    1
                } else {
                    16
                };

                let register_ammount = if ending_bit % register_size == 0 {
                    ending_bit / register_size
                } else {
                    ending_bit / register_size + 1
                };

                let mut needed_addresses = vec![];

                for i in 0..register_ammount {
                    needed_addresses.push(starting_address.address + i);
                }

                let binding = ValueBinding {
                    starting_address,
                    needed_addresses,
                    config: address.clone(),
                };

                value_bindings.push(binding);
            }
        }

        value_bindings
    }

    fn build_queries(config: &PolledConnection) -> Vec<Query> {
        let mut queries = vec![];

        for slave in &config.slaves {
            let mut table_divided_values: HashMap<ModbusTable, Vec<PolledValue>> = HashMap::new();

            for address in &slave.values {
                table_divided_values
                    .entry(address.table.to_tweakable_modbus_table())
                    .or_default()
                    .push(address.clone());
            }

            for vec in table_divided_values.values_mut() {
                vec.sort_by_key(|address| address.starting_address);
            }

            let max_gap = slave.config.max_gap_size_in_query as i64;
            let max_addresses_for_query = slave.config.max_register_ammount;

            for (table, values) in table_divided_values {
                let mut table_queries = vec![];
                let register_size =
                    if table == ModbusTable::Coils || table == ModbusTable::DiscreteInput {
                        1
                    } else {
                        16
                    };

                for value in values {
                    let ending_bit = value.starting_bit as u16 + value.bit_length;

                    let register_ammount = if ending_bit % register_size == 0 {
                        ending_bit / register_size
                    } else {
                        ending_bit / register_size + 1
                    };

                    let ending_register = value.starting_address + register_ammount - 1;

                    if !table_queries.is_empty() {
                        let last_query: &mut Query = table_queries.last_mut().unwrap();
                        let ammount = (ending_register - last_query.starting_address) + 1;

                        if (value.starting_address - last_query.ending_address) as i64 <= max_gap
                            && ammount as u32 <= max_addresses_for_query
                            && last_query.poll_time == value.poll_time
                        {
                            last_query.ending_address = ending_register;
                            last_query.ammount = ammount;
                            continue;
                        }
                    }

                    let query = Query {
                        starting_address: value.starting_address,
                        ending_address: ending_register,
                        ammount: register_ammount,
                        table,
                        slave_id: slave.id,
                        poll_time: value.poll_time,
                    };

                    table_queries.push(query);
                }
                queries.extend_from_slice(&table_queries);
            }
        }
        queries
    }
}
