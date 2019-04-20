use crate::parser::StructDefn;
use crate::type_check::TSchema;

pub struct Nugget {
    pub start: u32,
    pub len: u32,
    pub name: String,
    pub value: Option<String>,
    pub children: Vec<Nugget>,
}

pub fn apply_schema(schema: &TSchema, _file_data: &str) -> Nugget {
    // We know this struct must exist, as we checked for it during the correctness checks
    let root_struct = schema.types.get("root").unwrap();
    let start = 0;
    build_nugget(start, root_struct, schema)
}

fn build_nugget(start: u32, kind: &StructDefn, _schema: &TSchema) -> Nugget {
    Nugget {
        start,
        len: 0,
        name: kind.name.clone(),
        value: None,
        children: Vec::new(),
    }
}
