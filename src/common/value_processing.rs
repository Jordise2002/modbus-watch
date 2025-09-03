use crate::common::model::{DataType, Value, ValueFormattingParams};

use anyhow::{anyhow, Result};
use tweakable_modbus::ModbusDataType;

fn apply_endianness(
    registers: &Vec<ModbusDataType>,
    byte_swap: bool,
    word_swap: bool,
    double_word_swap: bool,
) -> Vec<u8> {
    let mut result = vec![];
    //All registers must be of the same type
    if let ModbusDataType::Coil(_) = registers.first().unwrap() {
        for register in registers {
            if let ModbusDataType::Coil(coil) = *register {
                result.push(coil as u8);
            }
        }
    } else {
        for register in registers {
            if let ModbusDataType::Register(register) = register {
                result.extend_from_slice(&register.to_le_bytes());
            }
        }
    }

    // Byte swap: intercambia cada par de bytes
    if byte_swap {
        let mut swapped = vec![];
        let mut iter = result.chunks_exact(2);
        while let Some([a, b]) = iter.next().map(|chunk| [chunk[0], chunk[1]]) {
            swapped.push(b);
            swapped.push(a);
        }
        swapped.extend_from_slice(iter.remainder());
        result = swapped;
    }

    // Word swap: intercambia cada par de palabras (4 bytes)
    if word_swap {
        let mut swapped = vec![];
        let mut iter = result.chunks_exact(4);
        while let Some(chunk) = iter.next() {
            if chunk.len() == 4 {
                swapped.extend_from_slice(&chunk[2..4]);
                swapped.extend_from_slice(&chunk[0..2]);
            }
        }
        swapped.extend_from_slice(iter.remainder());
        result = swapped;
    }

    // Double word swap: intercambia bloques de 4 palabras (8 bytes)
    if double_word_swap {
        let mut swapped = vec![];
        let mut iter = result.chunks_exact(8);
        while let Some(chunk) = iter.next() {
            if chunk.len() == 8 {
                swapped.extend_from_slice(&chunk[4..8]);
                swapped.extend_from_slice(&chunk[0..4]);
            }
        }
        swapped.extend_from_slice(iter.remainder());
        result = swapped;
    }

    result
}

fn apply_mask(data: &Vec<u8>, start_bit: usize, length: usize) -> Vec<u8> {
    let mut result = vec![];
    let mut current_byte = 0u8;
    let mut bit_pos = 0;

    for i in 0..length {
        let absolute_bit = start_bit + i;
        let byte_index = absolute_bit / 8;
        let bit_index = 7 - (absolute_bit % 8); // bits van de MSB a LSB

        if byte_index >= data.len() {
            break; // fuera de rango
        }

        let bit = (data[byte_index] >> bit_index) & 1;
        current_byte = (current_byte << 1) | bit;
        bit_pos += 1;

        if bit_pos == 8 {
            result.push(current_byte);
            current_byte = 0;
            bit_pos = 0;
        }
    }

    // Si quedan bits pendientes (menos de 8)
    if bit_pos > 0 {
        current_byte <<= 8 - bit_pos; // rellena con ceros al final
        result.push(current_byte);
    }

    result
}

pub fn registers_to_bytes(registers: Vec<ModbusDataType>, config: &ValueFormattingParams) -> Vec<u8> {
    let mut bytes = apply_endianness(
        &registers,
        config.byte_swap,
        config.word_swap,
        config.double_word_swap,
    );

    if config.data_type != DataType::Boolean
    {
        bytes = apply_mask(
            &bytes,
            config.starting_bit as usize,
            config.bit_length as usize,
        );
    }

    return bytes;
}

pub fn value_to_bytes(value: Value) -> Vec<u8> {
    match value {
        Value::Integer(integer) => integer.to_le_bytes().to_vec(),
        Value::FloatingPoint(floating) => floating.to_le_bytes().to_vec(),
        Value::Boolean(boolean) => {
            vec![boolean as u8]
        }
    }
}

pub fn value_to_registers(value: Value, config: &ValueFormattingParams) -> Vec<ModbusDataType>
{
    vec![]
}

pub fn format_value(raw_value: Vec<u8>, data_type: &DataType) -> Result<Value> {
    if raw_value.is_empty() {
        return Err(anyhow!("Value is empty"));
    }
    match data_type {
        DataType::Boolean => {
            if raw_value.len() != 1 {
                return Err(anyhow!("Boolean values must be one byte long"));
            }

            Ok(Value::Boolean(raw_value[0] != 0))
        }
        DataType::Float | DataType::Double => {
            if raw_value.len() != 8 as usize {
                return Err(anyhow!("Double values must be 8 bytes long"));
            }

            Ok(Value::FloatingPoint(f64::from_le_bytes(
                raw_value.try_into().unwrap(),
            )))
        }
        DataType::Byte => {
            let significant_bytes = &raw_value[..1];
            let byte_value = u8::from_le_bytes(significant_bytes.try_into().unwrap());

            Ok(Value::Integer(byte_value as i128))
        }
        DataType::SignedInteger16 => {
            let significant_bytes = &raw_value[..2];
            let signed_16_value = i16::from_le_bytes(significant_bytes.try_into().unwrap());

            Ok(Value::Integer(signed_16_value as i128))
        }
        DataType::SignedInteger32 => {
            let significant_bytes = &raw_value[..4];
            let signed_32_value = i32::from_le_bytes(significant_bytes.try_into().unwrap());

            Ok(Value::Integer(signed_32_value as i128))
        }
        DataType::SignedInteger64 => {
            let significant_bytes = &raw_value[..8];
            let signed_64_value = i64::from_le_bytes(significant_bytes.try_into().unwrap());

            Ok(Value::Integer(signed_64_value as i128))
        }
        DataType::UnsignedInteger16 => {
            let significant_bytes = &raw_value[..2];
            let unsigned_16_value = u16::from_le_bytes(significant_bytes.try_into().unwrap());

            Ok(Value::Integer(unsigned_16_value as i128))
        }
        DataType::UnsignedInteger32 => {
            let significant_bytes = &raw_value[..4];
            let unsigned_32_value = u32::from_le_bytes(significant_bytes.try_into().unwrap());

            Ok(Value::Integer(unsigned_32_value as i128))
        }
        DataType::UnsignedInteger64 => {
            let significant_bytes = &raw_value[..8];
            let unsigned_64_value = u64::from_le_bytes(significant_bytes.try_into().unwrap());

            Ok(Value::Integer(unsigned_64_value as i128))
        }
    }
}
