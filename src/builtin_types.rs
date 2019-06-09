use byteorder::{BigEndian, ByteOrder, LittleEndian};

struct CartaBuiltinType<'a> {
    size: usize,
    value: &'a Fn(&[u8]) -> String,
}

fn get_builtin_types(name: &str) -> Option<CartaBuiltinType<'static>> {
    match name {
        "int8" => Some(CartaBuiltinType {
            size: 1,
            value: &|data| i8::from_le_bytes([data[0]]).to_string(),
        }),
        "int16_be" => Some(CartaBuiltinType {
            size: 2,
            value: &|data| BigEndian::read_i16(data).to_string(),
        }),
        "int16_le" => Some(CartaBuiltinType {
            size: 2,
            value: &|data| LittleEndian::read_u16(data).to_string(),
        }),
        "int32_be" => Some(CartaBuiltinType {
            size: 4,
            value: &|data| BigEndian::read_i32(data).to_string(),
        }),
        "int32_le" => Some(CartaBuiltinType {
            size: 4,
            value: &|data| LittleEndian::read_u32(data).to_string(),
        }),
        "int64_be" => Some(CartaBuiltinType {
            size: 8,
            value: &|data| BigEndian::read_i64(data).to_string(),
        }),
        "int64_le" => Some(CartaBuiltinType {
            size: 8,
            value: &|data| LittleEndian::read_i64(data).to_string(),
        }),
        "uint8" => Some(CartaBuiltinType {
            size: 1,
            value: &|data| u8::from_le_bytes([data[0]]).to_string(),
        }),
        "uint16_be" => Some(CartaBuiltinType {
            size: 2,
            value: &|data| BigEndian::read_u16(data).to_string(),
        }),
        "uint16_le" => Some(CartaBuiltinType {
            size: 2,
            value: &|data| LittleEndian::read_u16(data).to_string(),
        }),
        "uint32_be" => Some(CartaBuiltinType {
            size: 4,
            value: &|data| BigEndian::read_u32(data).to_string(),
        }),
        "uint32_le" => Some(CartaBuiltinType {
            size: 4,
            value: &|data| LittleEndian::read_u32(data).to_string(),
        }),
        "uint64_be" => Some(CartaBuiltinType {
            size: 8,
            value: &|data| BigEndian::read_u64(data).to_string(),
        }),
        "uint64_le" => Some(CartaBuiltinType {
            size: 8,
            value: &|data| LittleEndian::read_u64(data).to_string(),
        }),
        "f32_be" => Some(CartaBuiltinType {
            size: 4,
            value: &|data| BigEndian::read_f32(data).to_string(),
        }),
        "f32_le" => Some(CartaBuiltinType {
            size: 4,
            value: &|data| LittleEndian::read_f32(data).to_string(),
        }),
        "f64_be" => Some(CartaBuiltinType {
            size: 8,
            value: &|data| BigEndian::read_f64(data).to_string(),
        }),
        "f64_le" => Some(CartaBuiltinType {
            size: 8,
            value: &|data| LittleEndian::read_f64(data).to_string(),
        }),
        _ => None,
    }
}

pub fn is_builtin_type(name: &str) -> bool {
    get_builtin_types(name).is_some()
}

pub fn get_value(data: &[u8], name: &str) -> Option<(usize, String)> {
    return get_builtin_types(name).map(|defn| (defn.size, (defn.value)(data)));
}
