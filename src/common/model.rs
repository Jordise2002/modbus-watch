use serde::{Deserialize, Serialize};

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

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Value {
    Integer(i128),
    FloatingPoint(f64),
    Boolean(bool),
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

    pub fn register_size(&self) -> usize {
        match self {
            ModbusTable::Coils | ModbusTable::DiscreteInput => 1,
            ModbusTable::HoldingRegisters | ModbusTable::InputRegisters => 16,
        }
    }
}

fn default_starting_bit() -> u8 {
    0
}

fn default_byte_swap() -> bool {
    false
}

fn default_word_swap() -> bool {
    false
}

fn default_double_word_swap() -> bool {
    false
}


#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ValueFormattingParams {
    #[serde(default = "default_starting_bit")]
    pub starting_bit: u8,
    pub bit_length: u16,

    pub data_type: DataType,

    #[serde(default = "default_byte_swap")]
    pub byte_swap: bool,
    #[serde(default = "default_word_swap")]
    pub word_swap: bool,
    #[serde(default = "default_double_word_swap")]
    pub double_word_swap: bool,
}
