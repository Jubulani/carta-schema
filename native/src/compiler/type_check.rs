use crate::compiler::parser::Schema;

pub struct Schema {
    nuggets: Vec<TNugget>,
    types: Vec<TNuggetStructDefn>,
}

struct TNugget {
    name: String,
    total_size: usize,
    kind: TNuggetTypeRef,
}

#[derive(PartialEq, Debug)]
enum TNuggetTypeRef {
    TNSimpleType(String)
    TNCompountType{
        struct_handle: usize
    }
}

struct TNuggetStructDefn {
    name: String,
    members: Vec<TNugget>
}

pub fn type_check_schema(schema: &mut Schema) {
}