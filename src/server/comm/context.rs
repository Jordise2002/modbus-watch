use std::net::IpAddr;
use std::sync::Arc;
use std::{collections::HashMap, net::SocketAddr};
use tweakable_modbus::{
    ExceptionCode, ModbusAddress, ModbusDataType, ModbusSlaveConnectionParameters,
};

use crate::server::model::connection::ServedConnectionConfig;
use crate::{
    common::model::{ModbusTable, ValueFormattingParams},
    server::{model::ServedConnection, state::AppState},
};

type AddressBindings = HashMap<ModbusAddress, String>;

struct ModbusSlaveCallback {
    app_state: AppState,
    bindings: AddressBindings
}

impl ModbusSlaveCallback {
    pub fn new(app_state: AppState, bindings: AddressBindings) -> Self {
        Self {app_state, bindings}
    }
}

#[async_trait::async_trait]
#[allow(dead_code)]
impl tweakable_modbus::ModbusCallBack for ModbusSlaveCallback {
    async fn on_read(&self, address: ModbusAddress) -> Result<ModbusDataType, ExceptionCode>
    {
        if !self.bindings.contains_key(&address) {
            return Err(ExceptionCode::IllegalDataAddress);
        }

        let value_id = self.bindings.get(&address).unwrap();

        let app_state_ref = self.app_state.lock().await;

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

    async fn on_write(&self, address: ModbusAddress, value: ModbusDataType) -> Result<(), ExceptionCode> {
        if !self.bindings.contains_key(&address) {
            return Err(ExceptionCode::IllegalDataAddress);
        }

        let value_id = self.bindings.get(&address).unwrap();

        let mut app_state_ref = self.app_state.lock().await;

        if !app_state_ref.contains_key(value_id) {
            return Err(ExceptionCode::ServerDeviceFailure);
        }

        let value_binding = app_state_ref.get_mut(value_id).unwrap();

        value_binding.set_register(address, value);

        Ok(())
    }

}
pub struct ModbusSlaveCommContext {
    address: SocketAddr,
    bindings: AddressBindings,
    app_state: AppState,
    config: ServedConnectionConfig,
}

impl ModbusSlaveCommContext {
    pub fn new(config: &ServedConnection, app_state: AppState) -> Arc<Self> {
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

        let address = SocketAddr::new(IpAddr::from([0, 0, 0, 0]), config.port);

        Arc::new(ModbusSlaveCommContext {
            address,
            bindings,
            app_state,
            config: config.config.clone(),
        })
    }

    pub fn serve(arc: Arc<Self>) {
        let callback = Box::new(ModbusSlaveCallback::new(arc.app_state.clone(), arc.bindings.clone()));

        let mut slave =
            tweakable_modbus::ModbusSlaveConnection::new_tcp(arc.address, callback);

        let params = ModbusSlaveConnectionParameters {
            connection_time_to_live: arc.config.connection_time_to_live,
            allowed_slaves: Arc::new(None),
            allowed_ip_address: None,
        };

        tokio::spawn(async move { slave.server_with_parameters(params).await.unwrap() });
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
