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
