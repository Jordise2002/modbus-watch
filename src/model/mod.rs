use serde::{Deserialize, Serialize};

mod value;
mod slave;
mod connection;

pub use value::PolledValue;
pub use slave::PolledSlave;
pub use connection::PolledConnection;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum DataType {
    Boolean,

    Byte,

    UnsignedInteger16,
    SignedInteger16,

    UnsignedInteger32,
    SignedInteger32,

    SignedInteger64,
    UnsignedInteger64,

    Float,
    Double,
}

impl DataType {
    //With integers we can always take the data as the LSBs but not with floating point values
    pub fn min_bit_size(&self) -> u16 {
        match self {
            DataType::Float => 32,
            DataType::Double => 64,
            _ => 1,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Endianness {
    BigEndian,
    LittleEndian,
    BigEndianByteSwapped,
    LittleEndianByteSwapped,
}

//I have to repeat this enum in order to use the derivation of serde traits :(
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Eq, Hash)]
pub enum ModbusTable {
    DiscreteInput,
    Coils,
    InputRegisters,
    HoldingRegisters,
}

impl ModbusTable {
    pub fn to_tweakable_modbus_table(&self) -> tweakable_modbus::ModbusTable {
        match self {
            ModbusTable::DiscreteInput => tweakable_modbus::ModbusTable::DiscreteInput,
            ModbusTable::Coils => tweakable_modbus::ModbusTable::Coils,
            ModbusTable::InputRegisters => tweakable_modbus::ModbusTable::InputRegisters,
            ModbusTable::HoldingRegisters => tweakable_modbus::ModbusTable::HoldingRegisters,
        }
    }
}
