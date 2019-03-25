/*!
 * Compiler - Compile .carta schema files into a usable intermediate representation that can be
 * applied to binary files.
 *
 * Stages of compilation:
 *
 * Tokenisation      Split the input file into Tokens
 *      |
 *      V
 *   Parsing         Extract file elements and structure definitions.  Returns a schema object
 *      |            that contains file elements (Nuggets), and NuggetStructDefns.
 *      V
 * Type checking     Uses the NuggetStructDefns and builtin types to do typechecking. Returns
 *      |            TNuggets (Typechecked Nuggets).
 *      V
 * Final representation
 */

mod builtin_types;
mod error;
mod parser;
mod tokeniser;
mod type_check;

use error::CartaError;
use type_check::TSchema;

pub fn compile_schema_file(data: &str) -> Result<TSchema, CartaError> {
    let tokeniser = tokeniser::Tokeniser::new(&data)?;
    let schema = parser::compile_schema(tokeniser)?;
    let tschema = type_check::type_check_schema(schema)?;
    Ok(tschema)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_basic_compile() {
        let res = compile_schema_file("new_name: int8");
        match res {
            Err(e) => panic!(e),
            Ok(_) => (),
        };
    }
}
