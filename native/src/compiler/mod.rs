mod tokeniser;

pub struct Schema {}

enum BuiltinTypes {
	Ascii,
	Int8,
	Int16LE,
	Int16BE,
	Int32LE,
	Int32BE,
}

enum NuggetType {
	UserDefinedType(),
	BuiltinType()
}

struct ILNugget {
	size: usize,
	name: String,
	kind: NuggetType,
	children: Vec<ILNugget>
}

pub fn compile_schema_file(filename: &str) -> Schema {
	let tok = tokeniser::load_file(filename);
	let _ = tok.iter().next();
	Schema {}
}
