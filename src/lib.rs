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
 * Correctness Checks  Final checks on the schema.  eg. Root element is correctly present.
 *      |
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

pub fn apply_schema(schema: &TSchema, file_data: &str) -> Nugget {
    apply::apply_schema(schema, file_data)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn basic_compile_and_apply() {
        let res = compile_schema_file("struct root {new_name: int8}");
        let schema = match res {
            Err(e) => panic!(format!("{}", e)),
            Ok(schema) => schema,
        };
        apply_schema(&schema, "This is some test data");
    }
}
