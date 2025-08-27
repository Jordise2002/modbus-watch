use anyhow::Result;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::mpsc::Sender;
use tokio::sync::Mutex;
use tracing::{debug, info, info_span, warn, Instrument};
use tweakable_modbus::{ModbusAddress, ModbusMasterConnection, ModbusResult, ModbusTable};

use crate::data::InsertValueMessage;
use crate::model::{PolledConnection, PolledValue};
use crate::value_processing;

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
    needed_ammount: u16,
    config: PolledValue,
}
pub struct ModbusCommContext {
    queries: Vec<Query>,
    value_bindings: Arc<HashMap<ModbusAddress, Vec<ValueBinding>>>,
    config: PolledConnection,
    insert_channel: Sender<InsertValueMessage>,
}

impl ModbusCommContext {
    pub fn new(config: PolledConnection, insert_channel: Sender<InsertValueMessage>) -> Self {
        let queries = Self::build_queries(&config);

        let value_bindings = Arc::new(Self::build_value_bindings(&config));

        ModbusCommContext {
            config,
            queries,
            value_bindings,
            insert_channel,
        }
    }

    fn load_queries(modbus_conn: &mut ModbusMasterConnection, queries: &Vec<Query>) {
        for query in queries {
            match query.table {
                ModbusTable::Coils => {
                    modbus_conn
                        .add_read_coils_query(query.slave_id, query.starting_address, query.ammount)
                        .unwrap();
                }
                ModbusTable::DiscreteInput => {
                    modbus_conn
                        .add_read_discrete_inputs_query(
                            query.slave_id,
                            query.starting_address,
                            query.ammount,
                        )
                        .unwrap();
                }
                ModbusTable::InputRegisters => {
                    modbus_conn
                        .add_read_input_registers_query(
                            query.slave_id,
                            query.starting_address,
                            query.ammount,
                        )
                        .unwrap();
                }
                ModbusTable::HoldingRegisters => {
                    modbus_conn
                        .add_read_holding_registers_query(
                            query.slave_id,
                            query.starting_address,
                            query.ammount,
                        )
                        .unwrap();
                }
            }
        }
    }

    async fn handle_results(
        results: HashMap<ModbusAddress, ModbusResult>,
        bindings: Arc<HashMap<ModbusAddress, Vec<ValueBinding>>>,
        tx: Sender<InsertValueMessage>,
    ) {
        for address in results.keys() {
            if !bindings.contains_key(address) {
                continue;
            }

            let address_bindings = bindings.get(address).unwrap();

            for address_binding in address_bindings {
                let mut value_registers = vec![];
                let mut address_pointer = address.clone();
                let mut success = true;

                for _i in 0..address_binding.needed_ammount {
                    if !results.contains_key(&address_pointer) {
                        success = false;
                        break;
                    }

                    if let ModbusResult::ReadResult(value) = results.get(&address_pointer).unwrap()
                    {
                        value_registers.push(value.clone());
                    } else {
                        success = false;
                        break;
                    }

                    address_pointer.address += 1;
                }

                if !success {
                    warn!(
                        "Registers were missing to build value {}",
                        address_binding.config.id
                    );
                    continue;
                }

                let value =
                    value_processing::registers_to_bytes(value_registers, &address_binding.config);

                let value = value_processing::format_value(value, &address_binding.config.data_type).unwrap();

                let value = value_processing::value_to_bytes(value);

                let insert = InsertValueMessage {
                    name: address_binding.config.id.clone(),
                    timestamp: std::time::SystemTime::now(),
                    value: value.clone(),
                };

                info!(
                    "Value {} received poll {:?}",
                    address_binding.config.id.clone(),
                    value
                );

                tx.send(insert).await.expect("Couldn't send message to db");
            }
        }
    }

    pub async fn query_loop(
        duration: std::time::Duration,
        queries: Vec<Query>,
        params: tweakable_modbus::ModbusMasterConnectionParams,
        master_connection: Arc<Mutex<ModbusMasterConnection>>,
        tx: Sender<InsertValueMessage>,
        bindings: Arc<HashMap<ModbusAddress, Vec<ValueBinding>>>,
    ) {
        let mut interval = tokio::time::interval(duration);

        loop {
            interval.tick().await;

            let mut modbus_conn = master_connection.lock().await;

            Self::load_queries(&mut modbus_conn, &queries);

            let results = modbus_conn.query_with_params(params).await;

            debug!("Modbus queries sent");

            if let Err(err) = results {
                warn!(
                    "Modbus query error: \"{}\", proceeding to next query",
                    err.to_string()
                );
                continue;
            }

            let results = results.unwrap();

            Self::handle_results(results, bindings.clone(), tx.clone()).await;
        }
    }

    pub async fn watch(&mut self) -> Result<()> {
        let mut queries_ordered_by_poll_time: HashMap<std::time::Duration, Vec<Query>> =
            HashMap::new();

        for query in &self.queries {
            queries_ordered_by_poll_time
                .entry(query.poll_time)
                .or_default()
                .push(query.clone());
        }

        let socket = SocketAddr::new(self.config.ip, self.config.port);
        let master_connection = Arc::new(Mutex::new(ModbusMasterConnection::new_tcp(socket)));

        let params = tweakable_modbus::ModbusMasterConnectionParams {
            max_response_time: self.config.config.max_response_time,
            max_simultaneous_transactions: self.config.config.max_simultaneous_connections,
        };

        let span = info_span!("Modbus connection", ip = %self.config.ip.to_string(), port = %self.config.port.to_string());

        for (interval, queries) in queries_ordered_by_poll_time {
            let master_connection = master_connection.clone();
            let bindings = self.value_bindings.clone();
            let tx = self.insert_channel.clone();

            tokio::task::spawn(
                async move {
                    Self::query_loop(
                        interval,
                        queries,
                        params.clone(),
                        master_connection,
                        tx,
                        bindings,
                    )
                    .await;
                }
                .instrument(span.clone()),
            );
        }
        Ok(())
    }

    fn build_value_bindings(
        config: &PolledConnection,
    ) -> HashMap<ModbusAddress, Vec<ValueBinding>> {
        let mut value_bindings: HashMap<ModbusAddress, Vec<ValueBinding>> = HashMap::new();

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

                let needed_ammount = if ending_bit % register_size == 0 {
                    ending_bit / register_size
                } else {
                    ending_bit / register_size + 1
                };

                let binding = ValueBinding {
                    needed_ammount,
                    config: address.clone(),
                };

                if value_bindings.contains_key(&starting_address) {
                    value_bindings
                        .get_mut(&starting_address)
                        .unwrap()
                        .push(binding);
                } else {
                    value_bindings.insert(starting_address, vec![binding]);
                }
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
