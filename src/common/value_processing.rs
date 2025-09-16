use crate::common::model::{DataType, Value, ValueFormattingParams};

use anyhow::{anyhow, Result};
use tweakable_modbus::ModbusDataType;

fn extract_bytes_from_registers(registers: &Vec<ModbusDataType>) -> Vec<u8> {
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
    result
}

fn build_registers_from_bytes(mut bytes: Vec<u8>) -> Vec<ModbusDataType> {
    if !bytes.len() % 2 == 0 {
        bytes.push(0);
    }

    bytes
        .chunks_exact(2)
        .map(|chunk| ModbusDataType::Register(u16::from_le_bytes([chunk[0], chunk[1]])))
        .collect()
}

fn apply_endianness(
    bytes: &Vec<u8>,
    byte_swap: bool,
    word_swap: bool,
    double_word_swap: bool,
) -> Vec<u8> {
    let mut result = bytes.clone();
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

fn apply_mask(mut data: Vec<u8>, start_bit: usize, length: usize) -> Vec<u8> {
    let mut result = vec![];
    let mut current_byte = 0u8;
    let mut bit_pos = 0;

    while data.len() * 8 < length
    {
        data.push(0);
    } 

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

fn move_to_mask_position(data: Vec<u8>, start_bit: usize, length: usize)  -> Vec<u8> {
    let mut result = vec![];

    let full_bytes_to_append = start_bit / 8;

    for i in 0..full_bytes_to_append {
        result.push(0);
    }

    let mut aux_byte = 0;
    let bit_index = start_bit % 8;

    for i in 0..length {
        let bit = (data[i / 8] >> i % 8) & 1;
        aux_byte = aux_byte | bit << bit_index;

        // Last bit
        if i % 8 == 7 {
            result.push(aux_byte);
            aux_byte = 0;
        }
    }

    result
}

pub fn registers_to_bytes(
    registers: Vec<ModbusDataType>,
    config: &ValueFormattingParams,
) -> Vec<u8> {
    let bytes = extract_bytes_from_registers(&registers);

    let mut bytes = apply_endianness(
        &bytes,
        config.byte_swap,
        config.word_swap,
        config.double_word_swap,
    );

    if config.data_type != DataType::Boolean {
        bytes = apply_mask(
            bytes,
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

pub fn value_to_registers(
    value: Value,
    config: &ValueFormattingParams,
) -> Result<Vec<ModbusDataType>> {
    let mut result = vec![];
    match config.data_type {
        DataType::Boolean => {
            if let Value::Boolean(value) = value {
                result.push(ModbusDataType::Coil(value));
            } else {
                return Err(anyhow!("Expected boolean value but found otherwise"));
            }
        }
        DataType::Double => {
            if let Value::FloatingPoint(value) = value {
                let bytes = value.to_le_bytes().to_vec();
                let bytes = apply_endianness(
                    &bytes,
                    config.byte_swap,
                    config.word_swap,
                    config.double_word_swap,
                );
                result = build_registers_from_bytes(bytes);
            }
            else {
                return Err(anyhow!("Expected floating point value but found otherwise"));
            }
        }
        DataType::Float => {
            if let Value::FloatingPoint(value) = value {
                let value = value as f32;
                let bytes = value.to_le_bytes().to_vec();
                let bytes = apply_endianness(
                    &bytes,
                    config.byte_swap,
                    config.word_swap,
                    config.double_word_swap,
                );
                result = build_registers_from_bytes(bytes);
            }
            else {
                return Err(anyhow!("Expected floating point value but found otherwise"));
            }
        }
        _ => {
            if let Value::Integer(value) = value {
                let value = value.to_le_bytes().to_vec();
                let value = value[..config.data_type.byte_size()].to_vec();

                let value = apply_endianness(&value, config.byte_swap, config.word_swap, config.double_word_swap);

                let value = move_to_mask_position(value, config.starting_bit as usize, config.bit_length as usize);

                result = build_registers_from_bytes(value)
            }
            else {
                return Err(anyhow!("Expected integer value but found otherwise"));
            }
        }
    }

    return Ok(result);
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
