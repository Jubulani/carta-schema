use serde_derive::Serialize;

use crate::builtin_types;
use crate::parser::{ElementTypeRef, StructDefn};
use crate::type_check::TSchema;

#[derive(PartialEq, Debug, Serialize)]
pub struct Nugget {
    pub start: usize,
    pub len: usize,
    pub name: String,
    pub value: Option<String>,
    pub children: Vec<Nugget>,
}

pub fn apply_schema(schema: &TSchema, file_data: &[u8]) -> Nugget {
    // We know this struct must exist, as we checked for it during the correctness checks
    let root_struct = schema.types.get("root").unwrap();
    let start = 0;
    build_nugget(start, root_struct, "root", schema, file_data)
}

fn build_nugget(start: usize, kind: &StructDefn, name: &str, schema: &TSchema, file_data: &[u8]) -> Nugget {
    let mut len = 0;

    let mut children = Vec::new();
    for element in &kind.elements {
        let ElementTypeRef::TypeName(typename) = &element.kind;
        if builtin_types::is_builtin_type(typename) {
            let elem_data = file_data.get(start + len..).unwrap();
            let (size, value) = builtin_types::get_value(elem_data, typename);
            let child = Nugget {
                start: start + len,
                len: size,
                name: element.name.clone(),
                value: Some(value),
                children: Vec::new(),
            };
            children.push(child);
            len += size;
        } else {
            // Must exist, as typechecking has passed for the schema
            let child_kind = schema.types.get(typename).unwrap();
            let child = build_nugget(start + len, child_kind, &element.name, schema, file_data);
            len += child.len;
            children.push(child);
        }
    }
    Nugget {
        start,
        len,
        name: name.to_string(),
        value: None,
        children,
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::compile_schema_file;

    #[test]
    fn u8_struct() {
        let schema =
            compile_schema_file("struct root {val1: int8, val2: int8, val3: int8}").unwrap();
        let res = apply_schema(&schema, b"\x00\x01\x02");
        assert_eq!(
            res,
            Nugget {
                start: 0,
                len: 3,
                name: "root".to_string(),
                value: None,
                children: vec![
                    Nugget {
                        start: 0,
                        len: 1,
                        name: "val1".to_string(),
                        value: Some("0".to_string()),
                        children: Vec::new(),
                    },
                    Nugget {
                        start: 1,
                        len: 1,
                        name: "val2".to_string(),
                        value: Some("1".to_string()),
                        children: Vec::new(),
                    },
                    Nugget {
                        start: 2,
                        len: 1,
                        name: "val3".to_string(),
                        value: Some("2".to_string()),
                        children: Vec::new(),
                    }
                ],
            }
        );
    }

    #[test]
    fn u8_i16_struct() {
        let schema =
            compile_schema_file("struct root {val1: int8, val2: int16_le, val3: int8}").unwrap();
        let res = apply_schema(&schema, b"\x00\x01\x00\x02");
        assert_eq!(
            res,
            Nugget {
                start: 0,
                len: 4,
                name: "root".to_string(),
                value: None,
                children: vec![
                    Nugget {
                        start: 0,
                        len: 1,
                        name: "val1".to_string(),
                        value: Some("0".to_string()),
                        children: Vec::new(),
                    },
                    Nugget {
                        start: 1,
                        len: 2,
                        name: "val2".to_string(),
                        value: Some("1".to_string()),
                        children: Vec::new(),
                    },
                    Nugget {
                        start: 3,
                        len: 1,
                        name: "val3".to_string(),
                        value: Some("2".to_string()),
                        children: Vec::new(),
                    }
                ],
            }
        );
    }

    #[test]
    fn child_structs() {
        let schema =
            compile_schema_file("struct root {version1: Version, version2: Version} struct Version {major: int8, minor: int8}").unwrap();
        let res = apply_schema(&schema, b"\x00\x01\x02\x03");
        assert_eq!(
            res,
            Nugget {
                start: 0,
                len: 4,
                name: "root".to_string(),
                value: None,
                children: vec![
                    Nugget {
                        start: 0,
                        len: 2,
                        name: "version1".to_string(),
                        value: None,
                        children: vec![
                            Nugget {
                                start: 0,
                                len: 1,
                                name: "major".to_string(),
                                value: Some("0".to_string()),
                                children: Vec::new(),
                            },
                            Nugget {
                                start: 1,
                                len: 1,
                                name: "minor".to_string(),
                                value: Some("1".to_string()),
                                children: Vec::new(),
                            }
                        ]
                    },
                    Nugget {
                        start: 2,
                        len: 2,
                        name: "version2".to_string(),
                        value: None,
                        children: vec![
                            Nugget {
                                start: 2,
                                len: 1,
                                name: "major".to_string(),
                                value: Some("2".to_string()),
                                children: Vec::new(),
                            },
                            Nugget {
                                start: 3,
                                len: 1,
                                name: "minor".to_string(),
                                value: Some("3".to_string()),
                                children: Vec::new(),
                            }
                        ]
                    }
                ]
            }
        );
    }
}
