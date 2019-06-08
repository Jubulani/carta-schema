/*!
 * Compiler - Compile .carta schema files into a usable internal representation that can be
 * applied to binary files.
 *
 * Stages of compilation:
 *
 * Tokenisation        Split the input file into Tokens
 *      |
 *      V
 *   Parsing           Extract file structure definitions.  Returns a schema object that contains
 *      |              a list of the structs, in the order they appeared in the input file.
 *      V
 * Type checking       Uses the StructDefns and builtin types to do type checking. Returns
 *      |              a tschema object with type checked types.
 *      V
 * Correctness Checks  Final checks on the schema.
 *      |               - Root element is correctly present
 *      |               - Array lengths can be calculated
 *      V
 * Final schema
 */

mod apply;
mod builtin_types;
mod correctness;
mod error;
mod parser;
mod tokeniser;
mod type_check;

pub use apply::Nugget;
use error::CartaError;
pub use type_check::TSchema;

pub fn compile_schema_file(data: &str) -> Result<TSchema, CartaError> {
    let tokeniser = tokeniser::Tokeniser::new(&data)?;
    let schema = parser::compile_schema(tokeniser)?;
    let tschema = type_check::type_check_schema(schema)?;
    correctness::check_schema(&tschema)?;
    Ok(tschema)
}

pub fn apply_schema(schema: &TSchema, file_data: &[u8]) -> Nugget {
    apply::apply_schema(schema, file_data)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn basic_compile_and_apply() {
        let schema = compile_schema_file("struct root {new_name: int8}").unwrap();
        apply_schema(&schema, b"\x00");
    }

    #[test]
    fn all_types() {
        let schema = compile_schema_file(
            "struct root {
                int8: int8,
                be: be,
                le: le
            }
            struct be {
                i_16: int16_be,
                i_32: int32_be,
                i_64: int64_be,
                u_16: uint16_be,
                u_32: uint32_be,
                u_64: uint64_be,
                f_32: f32_be,
                f_64: f64_be,
                arr: [int16_be; i_16]
            }
            struct le {
                int16: int16_le,
                int32: int32_le,
                int64: int64_le,
                uint16: uint16_le,
                uint32: uint32_le,
                uint64: uint64_le,
                f32: f32_le,
                f64: f64_le,
            }
        ",
        )
        .unwrap();
        apply_schema(&schema, &[0; 83]);
    }
}
