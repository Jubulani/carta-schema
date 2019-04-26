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
        "int8" => get_value_i8(data),
        "int16_be" => get_value_i16be(data),
        "int16_le" => get_value_i16le(data),
        "int32_be" => get_value_i32be(data),
        "int32_le" => get_value_i32le(data),
        "int64_be" => get_value_i64be(data),
        "int64_le" => get_value_i64le(data),
        "uint8" => get_value_u8(data),
        "uint16_be" => get_value_u16be(data),
        "uint16_le" => get_value_u16le(data),
        "uint32_be" => get_value_u32be(data),
        "uint32_le" => get_value_u32le(data),
        "uint64_be" => get_value_u64be(data),
        "uint64_le" => get_value_u64le(data),
        "f32_be" => (4, BigEndian::read_f32(data).to_string()),
        "f32_le" => (4, LittleEndian::read_f32(data).to_string()),
        "f64_be" => (8, BigEndian::read_f64(data).to_string()),
        "f64_le" => (8, LittleEndian::read_f64(data).to_string()),
        _ => panic!("Couldn't get value for type"),
    }
}

fn get_arr1(data: &[u8]) -> [u8; 1] {
    [data[0]]
}

fn get_arr2(data: &[u8]) -> [u8; 2] {
    [data[0], data[1]]
}

fn get_arr4(data: &[u8]) -> [u8; 4] {
    [data[0], data[1], data[2], data[3]]
}

fn get_arr8(data: &[u8]) -> [u8; 8] {
    [
        data[0], data[1], data[2], data[3], data[4], data[5], data[6], data[7],
    ]
}

fn get_value_i8(data: &[u8]) -> (usize, String) {
    assert!(std::mem::size_of::<i8>() == 1);
    let value = i8::from_le_bytes(get_arr1(data));
    (1, value.to_string())
}

fn get_value_i16be(data: &[u8]) -> (usize, String) {
    assert!(std::mem::size_of::<i16>() == 2);
    let value = i16::from_be_bytes(get_arr2(data));
    (2, value.to_string())
}

fn get_value_i16le(data: &[u8]) -> (usize, String) {
    assert!(std::mem::size_of::<i16>() == 2);
    let value = i16::from_le_bytes(get_arr2(data));
    (2, value.to_string())
}

fn get_value_i32be(data: &[u8]) -> (usize, String) {
    assert!(std::mem::size_of::<i32>() == 4);
    let value = i32::from_be_bytes(get_arr4(data));
    (4, value.to_string())
}

fn get_value_i32le(data: &[u8]) -> (usize, String) {
    assert!(std::mem::size_of::<i32>() == 4);
    let value = i32::from_le_bytes(get_arr4(data));
    (4, value.to_string())
}

fn get_value_i64be(data: &[u8]) -> (usize, String) {
    assert!(std::mem::size_of::<i64>() == 8);
    let value = i64::from_be_bytes(get_arr8(data));
    (8, value.to_string())
}

fn get_value_i64le(data: &[u8]) -> (usize, String) {
    assert!(std::mem::size_of::<i64>() == 8);
    let value = i64::from_le_bytes(get_arr8(data));
    (8, value.to_string())
}

fn get_value_u8(data: &[u8]) -> (usize, String) {
    assert!(std::mem::size_of::<u8>() == 1);
    let value = u8::from_le_bytes(get_arr1(data));
    (1, value.to_string())
}

fn get_value_u16be(data: &[u8]) -> (usize, String) {
    assert!(std::mem::size_of::<u16>() == 2);
    let value = u16::from_be_bytes(get_arr2(data));
    (2, value.to_string())
}

fn get_value_u16le(data: &[u8]) -> (usize, String) {
    assert!(std::mem::size_of::<u16>() == 2);
    let value = u16::from_le_bytes(get_arr2(data));
    (2, value.to_string())
}

fn get_value_u32be(data: &[u8]) -> (usize, String) {
    assert!(std::mem::size_of::<u32>() == 4);
    let value = u32::from_be_bytes(get_arr4(data));
    (4, value.to_string())
}

fn get_value_u32le(data: &[u8]) -> (usize, String) {
    assert!(std::mem::size_of::<u32>() == 4);
    let value = u32::from_le_bytes(get_arr4(data));
    (4, value.to_string())
}

fn get_value_u64be(data: &[u8]) -> (usize, String) {
    assert!(std::mem::size_of::<u64>() == 8);
    let value = u64::from_be_bytes(get_arr8(data));
    (8, value.to_string())
}

fn get_value_u64le(data: &[u8]) -> (usize, String) {
    assert!(std::mem::size_of::<u64>() == 8);
    let value = u64::from_le_bytes(get_arr8(data));
    (8, value.to_string())
}
