use std::sync::Arc;
use std::{collections::HashMap, net::SocketAddr};
use tweakable_modbus::{ExceptionCode, ModbusAddress, ModbusDataType};

use crate::{
    common::model::{ModbusTable, ValueFormattingParams},
    server::{model::ServedConnection, state::AppState},
};

type AddressBindings = HashMap<ModbusAddress, String>;

pub struct ModbusSlaveCommContext {
    pub address: SocketAddr,
    pub bindings: AddressBindings,
    pub app_state: AppState,
}

impl ModbusSlaveCommContext {
    pub fn new(config: &ServedConnection, app_state: AppState, address: SocketAddr) -> Arc<Self> {
        let mut bindings = AddressBindings::new();
        for slave in &config.slaves {
            for value in &slave.values {
                let registers = get_involved_addresses(
                    slave.id,
                    value.table.clone(),
                    value.starting_address,
                    &value.formatting_params,
                );

                for register in registers {
                    bindings.insert(register, value.id.clone());
                }
            }
        }

        Arc::new(ModbusSlaveCommContext {
            address,
            bindings,
            app_state,
        })
    }

    fn on_read(
        &self,
        address: ModbusAddress,
    ) -> std::result::Result<ModbusDataType, ExceptionCode> {
        if !self.bindings.contains_key(&address) {
            return Err(ExceptionCode::IllegalDataAddress);
        }

        let value_id = self.bindings.get(&address).unwrap();

        let app_state_ref = self.app_state.blocking_lock();

        if !app_state_ref.contains_key(value_id) {
            return Err(ExceptionCode::ServerDeviceFailure);
        }

        let value_binding = app_state_ref.get(value_id).unwrap();

        if let Some(value) = value_binding.get_register(address) {
            Ok(value)
        } else {
            Err(ExceptionCode::ServerDeviceFailure)
        }
    }

    fn on_write(
        &self,
        address: ModbusAddress,
        value: ModbusDataType,
    ) -> std::result::Result<(), ExceptionCode> {
        if !self.bindings.contains_key(&address) {
            return Err(ExceptionCode::IllegalDataAddress);
        }

        let value_id = self.bindings.get(&address).unwrap();

        let mut app_state_ref = self.app_state.blocking_lock();

        if !app_state_ref.contains_key(value_id) {
            return Err(ExceptionCode::ServerDeviceFailure);
        }

        let value_binding = app_state_ref.get_mut(value_id).unwrap();

        value_binding.set_register(address, value);

        Ok(())
    }

    pub async fn serve(arc: Arc<Self>) {
        let arc_read = arc.clone();

        let on_read = Box::new(move |address: ModbusAddress| arc_read.on_read(address));

        let arc_write = arc.clone();

        let on_write = Box::new(move |address: ModbusAddress, value: ModbusDataType| {
            arc_write.on_write(address, value)
        });

        let mut slave =
            tweakable_modbus::ModbusSlaveConnection::new_tcp(arc.address, on_read, on_write);

        slave.serve().await.unwrap();
    }
}

fn get_involved_addresses(
    slave_id: u8,
    table: ModbusTable,
    address: u16,
    formatting_params: &ValueFormattingParams,
) -> Vec<ModbusAddress> {
    let mut result = vec![];

    let mut starting_address = ModbusAddress {
        slave_id,
        table: table.to_tweakable_modbus_table(),
        address,
    };

    let total_bits = formatting_params.starting_bit as u16 + formatting_params.bit_length;

    let bits_in_register = table.register_size() as u16;

    let register_ammount = if total_bits % bits_in_register == 0 {
        total_bits / bits_in_register
    } else {
        (total_bits / bits_in_register) + 1
    };

    for _i in 0..register_ammount {
        result.push(starting_address.clone());
        starting_address.address += 1;
    }

    result
}
