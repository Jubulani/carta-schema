mod tokeniser;

pub struct Schema {}

pub fn compile_schema_file(filename: &str) -> Schema {
	let mut tok = tokeniser::load_file(filename);
	let _ = tok.next();
	Schema {}
}
