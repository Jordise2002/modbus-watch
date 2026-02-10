use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use tweakable_modbus::{ModbusAddress, ModbusDataType};

use crate::{
    common::{
        model::Value,
        value_processing,
    },
    server::model::{ServedConnection, ServedValue},
};

pub type AppState = Arc<Mutex<HashMap<String, ValueState>>>;

pub struct ValueState {
    pub starting_address: ModbusAddress,
    registers: Vec<ModbusDataType>,
    pub config: ServedValue,
}

impl ValueState {
    pub fn new(
        starting_address: ModbusAddress,
        default_value: Value,
        config: ServedValue,
    ) -> Self {
        let registers = value_processing::value_to_registers(default_value, &config.formatting_params).unwrap();

        ValueState {
            starting_address,
            registers,
            config,
        }
    }

    pub fn get_all_registers(&self) -> Vec<ModbusDataType> {
        self.registers.clone()
    }

    pub fn set_all_registers(&mut self, new_registers: Vec<ModbusDataType>) {
        self.registers = new_registers;
    }

    pub fn get_register(&self, address: ModbusAddress) -> Option<ModbusDataType> {
        if address.address < self.starting_address.address {
            return None;
        }

        let offset= (address.address - self.starting_address.address) as usize;

        if offset >= self.registers.len() {
            return None;
        }

        let value = self.registers[offset];

        Some(value)
    }

    pub fn set_register(& mut self, address: ModbusAddress, value:ModbusDataType) {
        if address.address < self.starting_address.address {
            return;
        }

        let offset = (address.address - self.starting_address.address) as usize;

        if offset >= self.registers.len() {
            return;
        }

        self.registers[offset] = value;
    }

}

pub fn build_app_state(config: &Vec<ServedConnection>) -> AppState {
    let mut app_state = HashMap::new();

    for connection in config {
        for slave in &connection.slaves {
            for value in &slave.values {
                let address = ModbusAddress {
                    slave_id: slave.id,
                    table: value.table.to_tweakable_modbus_table(),
                    address: value.starting_address,
                };

                app_state.insert(
                    value.id.clone(),
                    ValueState::new(
                        address,
                        value.default_value,
                        value.clone(),
                    ),
                );
            }
        }
    }
    Arc::new(Mutex::new(app_state))
}
