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
mod parser;
mod tokeniser;
mod type_check;

use std::fs;

pub fn compile_schema_file(filename: &str) {
    let s = fs::read_to_string(filename).unwrap();
    let tokeniser = tokeniser::Tokeniser::new(&s);
    let schema = parser::compile_schema(&tokeniser);
    type_check::type_check_schema(schema);
}
