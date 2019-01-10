mod parser;
mod tokeniser;
mod types;

use std::fs;

pub fn compile_schema_file(filename: &str) -> parser::Schema {
    let s = fs::read_to_string(filename).unwrap();
    parser::compile_schema(&s)
}

