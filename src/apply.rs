use serde_derive::Serialize;

use crate::builtin_types;
use crate::parser;
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

fn build_nugget(
    start: usize,
    struct_defn: &StructDefn,
    name: &str,
    schema: &TSchema,
    file_data: &[u8],
) -> Nugget {
    let mut len = 0;

    let mut children = Vec::new();
    for element in &struct_defn.elements {
        let (nugget, size) = match &element.kind {
            ElementTypeRef::TypeName(typename) => {
                build_single_val(typename, start + len, file_data, &element.name, schema)
            }
            ElementTypeRef::ArrayElem(array_defn) => build_array_val(
                array_defn,
                start + len,
                file_data,
                &element.name,
                schema,
                &children,
            ),
        };
        len += size;
        children.push(nugget);
    }
    Nugget {
        start,
        len,
        name: name.to_string(),
        value: None,
        children,
    }
}

fn build_single_val(
    typename: &str,
    start: usize,
    file_data: &[u8],
    name: &str,
    schema: &TSchema,
) -> (Nugget, usize) {
    if builtin_types::is_builtin_type(typename) {
        let elem_data = file_data.get(start..).unwrap();

        // is_builtin_type returned true, so this value must exist
        let (size, value) = builtin_types::get_value(elem_data, typename).unwrap();
        let child = Nugget {
            start: start,
            len: size,
            name: name.to_string(),
            value: Some(value),
            children: Vec::new(),
        };
        (child, size)
    } else {
        // Must exist, as typechecking has passed for the schema
        let child_kind = schema.types.get(typename).unwrap();
        let child = build_nugget(start, child_kind, name, schema, file_data);
        let len = child.len;
        (child, len)
    }
}

fn build_array_val(
    array_defn: &parser::ArrayDefn,
    start: usize,
    file_data: &[u8],
    name: &str,
    schema: &TSchema,
    siblings: &Vec<Nugget>,
) -> (Nugget, usize) {
    // Get the array len
    let len = get_elem_size_value(&array_defn.len_identifier, siblings).unwrap();

    let mut children = Vec::new();
    let mut size = 0;
    for i in 0..len {
        let child_name = i.to_string();
        let (child, len) = build_single_val(
            &array_defn.kind,
            start + size,
            file_data,
            &child_name,
            schema,
        );
        children.push(child);
        size += len;
    }
    (
        Nugget {
            start,
            len: size,
            name: name.to_string(),
            value: None,
            children,
        },
        size,
    )
}

fn get_elem_size_value(name: &str, nuggets: &Vec<Nugget>) -> Option<usize> {
    // Simple linear search among sibling nuggets
    for nugget in nuggets {
        if nugget.name == name {
            let value = &nugget.value.as_ref().unwrap();
            return Some(value.parse::<usize>().unwrap());
        }
    }
    None
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

    #[test]
    fn array() {
        let schema = compile_schema_file("struct root {len: int8, arr: [uint8; len]}").unwrap();
        let res = apply_schema(&schema, b"\x02\x00\x01");
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
                        name: "len".to_string(),
                        value: Some("2".to_string()),
                        children: Vec::new(),
                    },
                    Nugget {
                        start: 1,
                        len: 2,
                        name: "arr".to_string(),
                        value: None,
                        children: vec![
                            Nugget {
                                start: 1,
                                len: 1,
                                name: "0".to_string(),
                                value: Some("0".to_string()),
                                children: Vec::new(),
                            },
                            Nugget {
                                start: 2,
                                len: 1,
                                name: "1".to_string(),
                                value: Some("1".to_string()),
                                children: Vec::new(),
                            }
                        ],
                    }
                ]
            }
        );

        // Same schema, zero length array
        let res = apply_schema(&schema, b"\x00");
        assert_eq!(
            res,
            Nugget {
                start: 0,
                len: 1,
                name: "root".to_string(),
                value: None,
                children: vec![
                    Nugget {
                        start: 0,
                        len: 1,
                        name: "len".to_string(),
                        value: Some("0".to_string()),
                        children: Vec::new(),
                    },
                    Nugget {
                        start: 1,
                        len: 0,
                        name: "arr".to_string(),
                        value: None,
                        children: Vec::new(),
                    }
                ]
            }
        );
    }
}
