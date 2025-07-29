use crate::model::PolledValue;
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
                result.extend_from_slice(&register.to_be_bytes());
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

pub fn format_value(registers: Vec<ModbusDataType>, config: &PolledValue) -> Vec<u8> {
    let bytes = apply_endianness(&registers, config.byte_swap, config.word_swap, config.double_word_swap);

    let bytes = apply_mask(&bytes, config.starting_bit as usize, config.bit_length as usize);

    return bytes
}
