use std::convert::TryFrom;

#[derive(PartialEq)]
pub enum BuiltinTypeClass {
    Integer,
    Float,
    Text,
}

struct CartaBuiltinType<'a> {
    size: usize,
    value: &'a dyn Fn(&[u8]) -> String,
    class: BuiltinTypeClass,
}

fn to_arr<const SIZE: usize>(data: &[u8]) -> &[u8; SIZE] {
    <&[u8; SIZE]>::try_from(&data[0..SIZE]).unwrap()
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
            value: &|data| i16::from_be_bytes(*to_arr::<2>(data)).to_string(),
            class: BuiltinTypeClass::Integer,
        }),
        "int16_le" => Some(CartaBuiltinType {
            size: 2,
            value: &|data| i16::from_le_bytes(*to_arr::<2>(data)).to_string(),
            class: BuiltinTypeClass::Integer,
        }),
        "int32_be" => Some(CartaBuiltinType {
            size: 4,
            value: &|data| i32::from_be_bytes(*to_arr::<4>(data)).to_string(),
            class: BuiltinTypeClass::Integer,
        }),
        "int32_le" => Some(CartaBuiltinType {
            size: 4,
            value: &|data| i32::from_le_bytes(*to_arr::<4>(data)).to_string(),
            class: BuiltinTypeClass::Integer,
        }),
        "int64_be" => Some(CartaBuiltinType {
            size: 8,
            value: &|data| i64::from_be_bytes(*to_arr::<8>(data)).to_string(),
            class: BuiltinTypeClass::Integer,
        }),
        "int64_le" => Some(CartaBuiltinType {
            size: 8,
            value: &|data| i64::from_le_bytes(*to_arr::<8>(data)).to_string(),
            class: BuiltinTypeClass::Integer,
        }),
        "uint8" => Some(CartaBuiltinType {
            size: 1,
            value: &|data| u8::from_le_bytes([data[0]]).to_string(),
            class: BuiltinTypeClass::Integer,
        }),
        "uint16_be" => Some(CartaBuiltinType {
            size: 2,
            value: &|data| u16::from_be_bytes(*to_arr::<2>(data)).to_string(),
            class: BuiltinTypeClass::Integer,
        }),
        "uint16_le" => Some(CartaBuiltinType {
            size: 2,
            value: &|data| u16::from_le_bytes(*to_arr::<2>(data)).to_string(),
            class: BuiltinTypeClass::Integer,
        }),
        "uint32_be" => Some(CartaBuiltinType {
            size: 4,
            value: &|data| u32::from_be_bytes(*to_arr::<4>(data)).to_string(),
            class: BuiltinTypeClass::Integer,
        }),
        "uint32_le" => Some(CartaBuiltinType {
            size: 4,
            value: &|data| u32::from_le_bytes(*to_arr::<4>(data)).to_string(),
            class: BuiltinTypeClass::Integer,
        }),
        "uint64_be" => Some(CartaBuiltinType {
            size: 8,
            value: &|data| u64::from_be_bytes(*to_arr::<8>(data)).to_string(),
            class: BuiltinTypeClass::Integer,
        }),
        "uint64_le" => Some(CartaBuiltinType {
            size: 8,
            value: &|data| u64::from_le_bytes(*to_arr::<8>(data)).to_string(),
            class: BuiltinTypeClass::Integer,
        }),
        "f32_be" => Some(CartaBuiltinType {
            size: 4,
            value: &|data| f32::from_be_bytes(*to_arr::<4>(data)).to_string(),
            class: BuiltinTypeClass::Float,
        }),
        "f32_le" => Some(CartaBuiltinType {
            size: 4,
            value: &|data| f32::from_le_bytes(*to_arr::<4>(data)).to_string(),
            class: BuiltinTypeClass::Float,
        }),
        "f64_be" => Some(CartaBuiltinType {
            size: 8,
            value: &|data| f64::from_be_bytes(*to_arr::<8>(data)).to_string(),
            class: BuiltinTypeClass::Float,
        }),
        "f64_le" => Some(CartaBuiltinType {
            size: 8,
            value: &|data| f64::from_le_bytes(*to_arr::<8>(data)).to_string(),
            class: BuiltinTypeClass::Float,
        }),
        // Single ascii character
        "ascii" => Some(CartaBuiltinType {
            size: 1,
            value: &|data| (u8::from_le_bytes([data[0]]) as char).to_string(),
            class: BuiltinTypeClass::Text,
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

pub fn is_type_class(name: &str, class: BuiltinTypeClass) -> bool {
    get_builtin_types(name)
        .map(|defn| defn.class == class)
        .unwrap_or(false)
}
