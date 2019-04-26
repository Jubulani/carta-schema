use byteorder::{BigEndian, ByteOrder, LittleEndian};

pub fn get_type(name: &str) -> Option<usize> {
    match name {
        "int8" => Some(1),
        "int16_be" => Some(2),
        "int16_le" => Some(2),
        "int32_be" => Some(4),
        "int32_le" => Some(4),
        "int64_be" => Some(8),
        "int64_le" => Some(8),
        "uint8" => Some(1),
        "uint16_be" => Some(2),
        "uint16_le" => Some(2),
        "uint32_be" => Some(4),
        "uint32_le" => Some(4),
        "uint64_be" => Some(8),
        "uint64_le" => Some(8),
        "f32_be" => Some(4),
        "f32_le" => Some(4),
        "f64_be" => Some(8),
        "f64_le" => Some(8),
        _ => None,
    }
}

pub fn is_builtin_type(name: &str) -> bool {
    get_type(name).is_some()
}

pub fn get_value(data: &[u8], type_name: &str) -> (usize, String) {
    match type_name {
        "int8" => (1, i8::from_le_bytes([data[0]]).to_string()),
        "int16_be" => (2, BigEndian::read_i16(data).to_string()),
        "int16_le" => (2, LittleEndian::read_u16(data).to_string()),
        "int32_be" => (4, BigEndian::read_i32(data).to_string()),
        "int32_le" => (4, LittleEndian::read_u32(data).to_string()),
        "int64_be" => (8, BigEndian::read_i64(data).to_string()),
        "int64_le" => (8, LittleEndian::read_i64(data).to_string()),
        "uint8" => (1, u8::from_le_bytes([data[0]]).to_string()),
        "uint16_be" => (2, BigEndian::read_u16(data).to_string()),
        "uint16_le" => (2, LittleEndian::read_u16(data).to_string()),
        "uint32_be" => (4, BigEndian::read_u32(data).to_string()),
        "uint32_le" => (4, LittleEndian::read_u32(data).to_string()),
        "uint64_be" => (8, BigEndian::read_u64(data).to_string()),
        "uint64_le" => (8, LittleEndian::read_u64(data).to_string()),
        "f32_be" => (4, BigEndian::read_f32(data).to_string()),
        "f32_le" => (4, LittleEndian::read_f32(data).to_string()),
        "f64_be" => (8, BigEndian::read_f64(data).to_string()),
        "f64_le" => (8, LittleEndian::read_f64(data).to_string()),
        _ => panic!("Couldn't get value for type"),
    }
}
