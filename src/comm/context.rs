use std::{collections::HashMap, ops::Add, task::Poll};

use tweakable_modbus::{ModbusDataType, ModbusTable};

use crate::{
    comm::AddressAndFunction,
    model::{PolledConnection, PolledSlave, PolledValue},
};

type SlaveGrid = HashMap<AddressAndFunction, ModbusDataType>;

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Query {
    pub starting_address: u16,
    pub ending_address: u16,
    pub ammount: u16,
    pub table: ModbusTable,
    pub slave_id: u8,
}

pub struct ModbusCommContext {
    queries: Vec<Query>,
    slave_grids: HashMap<u8, SlaveGrid>,
    config: PolledConnection,
}

impl ModbusCommContext {
    pub fn new(config: PolledConnection) -> Self {
        let slave_grids = HashMap::new();

        let queries = Self::build_queries(&config);

        ModbusCommContext {
            slave_grids,
            config,
            queries,
        }
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

            let max_gap = slave.config.max_gap_size_in_query;

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

                        if (value.starting_address - last_query.ending_address) as u32 <= max_gap {
                            last_query.ending_address = ending_register;
                            last_query.ammount =
                                (ending_register - last_query.starting_address) + 1;
                            continue;
                        }
                    }

                    let query = Query {
                        starting_address: value.starting_address,
                        ending_address: ending_register,
                        ammount: register_ammount,
                        table,
                        slave_id: slave.id,
                    };

                    table_queries.push(query);
                }
                queries.extend_from_slice(&table_queries);
            }
        }
        queries
    }
}
