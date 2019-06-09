use byteorder::{BigEndian, ByteOrder, LittleEndian};

#[derive(PartialEq)]
enum BuiltinTypeClass {
    Integer,
    Float,
}

struct CartaBuiltinType<'a> {
    size: usize,
    value: &'a Fn(&[u8]) -> String,
    class: BuiltinTypeClass,
}

fn get_builtin_types(name: &str) -> Option<CartaBuiltinType<'static>> {
    match name {
        "int8" => Some(CartaBuiltinType {
            size: 1,
            value: &|data| i8::from_le_bytes([data[0]]).to_string(),
            class: BuiltinTypeClass::Integer,
        }),
        "int16_be" => Some(CartaBuiltinType {
            size: 2,
            value: &|data| BigEndian::read_i16(data).to_string(),
            class: BuiltinTypeClass::Integer,
        }),
        "int16_le" => Some(CartaBuiltinType {
            size: 2,
            value: &|data| LittleEndian::read_u16(data).to_string(),
            class: BuiltinTypeClass::Integer,
        }),
        "int32_be" => Some(CartaBuiltinType {
            size: 4,
            value: &|data| BigEndian::read_i32(data).to_string(),
            class: BuiltinTypeClass::Integer,
        }),
        "int32_le" => Some(CartaBuiltinType {
            size: 4,
            value: &|data| LittleEndian::read_u32(data).to_string(),
            class: BuiltinTypeClass::Integer,
        }),
        "int64_be" => Some(CartaBuiltinType {
            size: 8,
            value: &|data| BigEndian::read_i64(data).to_string(),
            class: BuiltinTypeClass::Integer,
        }),
        "int64_le" => Some(CartaBuiltinType {
            size: 8,
            value: &|data| LittleEndian::read_i64(data).to_string(),
            class: BuiltinTypeClass::Integer,
        }),
        "uint8" => Some(CartaBuiltinType {
            size: 1,
            value: &|data| u8::from_le_bytes([data[0]]).to_string(),
            class: BuiltinTypeClass::Integer,
        }),
        "uint16_be" => Some(CartaBuiltinType {
            size: 2,
            value: &|data| BigEndian::read_u16(data).to_string(),
            class: BuiltinTypeClass::Integer,
        }),
        "uint16_le" => Some(CartaBuiltinType {
            size: 2,
            value: &|data| LittleEndian::read_u16(data).to_string(),
            class: BuiltinTypeClass::Integer,
        }),
        "uint32_be" => Some(CartaBuiltinType {
            size: 4,
            value: &|data| BigEndian::read_u32(data).to_string(),
            class: BuiltinTypeClass::Integer,
        }),
        "uint32_le" => Some(CartaBuiltinType {
            size: 4,
            value: &|data| LittleEndian::read_u32(data).to_string(),
            class: BuiltinTypeClass::Integer,
        }),
        "uint64_be" => Some(CartaBuiltinType {
            size: 8,
            value: &|data| BigEndian::read_u64(data).to_string(),
            class: BuiltinTypeClass::Integer,
        }),
        "uint64_le" => Some(CartaBuiltinType {
            size: 8,
            value: &|data| LittleEndian::read_u64(data).to_string(),
            class: BuiltinTypeClass::Integer,
        }),
        "f32_be" => Some(CartaBuiltinType {
            size: 4,
            value: &|data| BigEndian::read_f32(data).to_string(),
            class: BuiltinTypeClass::Float,
        }),
        "f32_le" => Some(CartaBuiltinType {
            size: 4,
            value: &|data| LittleEndian::read_f32(data).to_string(),
            class: BuiltinTypeClass::Float,
        }),
        "f64_be" => Some(CartaBuiltinType {
            size: 8,
            value: &|data| BigEndian::read_f64(data).to_string(),
            class: BuiltinTypeClass::Float,
        }),
        "f64_le" => Some(CartaBuiltinType {
            size: 8,
            value: &|data| LittleEndian::read_f64(data).to_string(),
            class: BuiltinTypeClass::Float,
        }),
        _ => None,
    }
}

pub fn is_builtin_type(name: &str) -> bool {
    get_builtin_types(name).is_some()
}

pub fn get_value(data: &[u8], name: &str) -> Option<(usize, String)> {
    get_builtin_types(name).map(|defn| (defn.size, (defn.value)(data)))
}

pub fn is_integer_type(name: &str) -> bool {
    get_builtin_types(name)
        .map(|defn| defn.class == BuiltinTypeClass::Integer)
        .unwrap_or(false)
}
