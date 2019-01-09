pub fn get_type(name: &str) -> (usize, &'static str) {
    match name {
        "int8" => (1, "int8"),
        "int16_be" => (2, "int16_be"),
        "int16_le" => (2, "int16_le"),
        "int32_be" => (4, "int32_be"),
        "int32_le" => (4, "int32_le"),
        "int64_be" => (8, "int64_be"),
        "int64_le" => (8, "int64_le"),
        "uint8" => (1, "uint8"),
        "uint16_be" => (2, "uint16_be"),
        "uint16_le" => (2, "uint16_le"),
        "uint32_be" => (4, "uint32_be"),
        "uint32_le" => (4, "uint32_le"),
        "uint64_be" => (8, "uint64_be"),
        "uint64_le" => (8, "uint64_le"),
        "f32_be" => (4, "f32_be"),
        "f32_le" => (4, "f32_le"),
        "f64_be" => (8, "f64_be"),
        "f64_le" => (8, "f64_le"),
        _ => panic!("Unrecognised type: {}", name),
    }
}
