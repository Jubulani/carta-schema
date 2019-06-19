use std::collections::{HashMap, HashSet};

use crate::builtin_types;
use crate::error::CartaError;
use crate::parser::{ElementTypeRef, Schema, StructDefn};

#[derive(PartialEq, Debug)]
pub struct TSchema {
    pub types: HashMap<String, StructDefn>,
}

pub fn type_check_schema(schema: Schema) -> Result<TSchema, CartaError> {
    let types = check_types(schema.structs)?;
    Ok(TSchema { types })
}

fn build_structs_map(types: Vec<StructDefn>) -> Result<HashMap<String, StructDefn>, CartaError> {
    let mut types_map: HashMap<String, StructDefn> = HashMap::new();

    for kind in types.into_iter() {
        if types_map.contains_key::<str>(&kind.name) {
            return Err(CartaError::new_duplicate_type(0, kind.name));
        }
        types_map.insert(kind.name.clone(), kind);
    }

    Ok(types_map)
}

fn check_all_types_defined(types_map: &HashMap<String, StructDefn>) -> Result<(), CartaError> {
    // All types are now stored in types_map.  We can now go over all members of all types, and
    // check that they've all been defined.
    for kind in types_map.values() {
        for member in &kind.elements {
            let typename = match &member.kind {
                ElementTypeRef::TypeName(typename) => &typename,
                ElementTypeRef::ArrayElem(array_defn) => &array_defn.kind,
            };
            //let ElementTypeRef::TypeName(typename) = &member.kind;
            if !builtin_types::is_builtin_type(&typename)
                && types_map.get::<str>(&typename).is_none()
            {
                return Err(CartaError::new_unknown_type(0, typename.to_string()));
            }
        }
    }

    Ok(())
}

/// Check that there are no types that recursively depend on themselves.
fn check_types_no_loops(types_map: &HashMap<String, StructDefn>) -> Result<(), CartaError> {
    // Set of all types that have been fully resolved to depend only on builtin types, or
    // other types that depend transitively on only built-in types.
    // Hopefully we can eventually add all the types to this set.  If we can't, there must be a loop
    let mut types_resolved: HashSet<&str> = HashSet::new();

    // Map of types to a list of types that depend on this type
    let mut dependant_types: HashMap<&str, Vec<&str>> = HashMap::new();

    // Once a type has been determined to depend (transitively) on only builtin types, then we
    // can check all types that depend on this type to see if they now do also (using the
    // dependant_types map).  This is the stack of types to check.
    let mut types_stack: Vec<&str> = Vec::new();

    // Build the dependant_types map.  Types that trivially depend only on builtin types can be
    // detected here as well.
    for kind in types_map.values() {
        let mut all_builtin = true;
        for member in &kind.elements {
            let typename = match &member.kind {
                ElementTypeRef::TypeName(typename) => &typename,
                ElementTypeRef::ArrayElem(array_defn) => &array_defn.kind,
            };
            if !builtin_types::is_builtin_type(&typename)
                && types_resolved.get::<str>(&typename).is_none()
            {
                all_builtin = false;

                if !dependant_types.contains_key::<str>(&typename) {
                    dependant_types.insert(&typename, Vec::new());
                }
                dependant_types
                    .get_mut::<str>(&typename)
                    .unwrap()
                    .push(&kind.name);
            }
        }

        if all_builtin {
            types_resolved.insert(&kind.name);
            types_stack.push(&kind.name);
        }
    }

    // Go over the stack of resolved types.  Use the dependant_types map to check if the types
    // the depend on the resolved type can be marked as resolved.
    while let Some(kind_name) = types_stack.pop() {
        if let Some(parents) = dependant_types.get::<str>(&kind_name) {
            for parent in parents.iter() {
                let parent = match types_map.get::<str>(parent) {
                    Some(p) => p,
                    // Should not be possible for this to happen, as we've previously called
                    // check_all_types_defined to check that all types are known.
                    None => panic!("Unresolved type: {:?}", parent),
                };

                let mut all_resolved = true;
                for member in &parent.elements {
                    let typename = match &member.kind {
                        ElementTypeRef::TypeName(typename) => &typename,
                        ElementTypeRef::ArrayElem(array_defn) => &array_defn.kind,
                    };
                    if !builtin_types::is_builtin_type(&typename)
                        && types_resolved.get::<str>(&typename).is_none()
                    {
                        all_resolved = false;
                    }
                }

                // Type is now fully resolved.
                if all_resolved {
                    types_resolved.insert(&parent.name);
                    types_stack.push(&parent.name);
                }
            }
        }
    }

    // If any types remain that aren't listed in types_resolved, then we must have a loop
    let mut recursive_types = Vec::new();
    for kind in types_map.values() {
        if types_resolved.get::<str>(&kind.name).is_none() {
            recursive_types.push(kind.name.clone());
        }
    }
    if !recursive_types.is_empty() {
        return Err(CartaError::new_recursive_types(0, recursive_types));
    }

    Ok(())
}

fn check_types(types: Vec<StructDefn>) -> Result<HashMap<String, StructDefn>, CartaError> {
    let types_map = build_structs_map(types)?;
    check_all_types_defined(&types_map)?;
    check_types_no_loops(&types_map)?;

    Ok(types_map)
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::parser::Element;
    use std::fmt::Debug;
    use crate::error::CartaErrorCode;

    fn build_element(name: &str, typename: &str) -> Element {
        Element {
            name: name.to_string(),
            kind: ElementTypeRef::TypeName(typename.to_string()),
        }
    }

    fn build_struct(name: &str, elements: Vec<Element>) -> StructDefn {
        StructDefn {
            name: name.to_string(),
            elements: elements,
        }
    }

    // Compare two vectors, where element order doesn't matter
    fn compare_vec_unordered<T: PartialEq + Debug>(a: Vec<T>, b: Vec<T>) {
        if a.len() != b.len() {
            panic!("{:?} != {:?}", &a, &b);
        }
        // Assumes elements in a are unique
        for elem in &a {
            if !b.contains(&elem) {
                panic!("{:?} != {:?}", a, b);
            }
        }
    }

    #[test]
    fn basic_ok() -> Result<(), CartaError> {
        let elem1 = build_element("inner1", "uint16_le");
        let schema = Schema {
            structs: vec![build_struct("type1", vec![elem1])],
        };
        type_check_schema(schema)?;
        Ok(())
    }

    #[test]
    fn multiple_structs() -> Result<(), CartaError> {
        let t1 = build_struct(
            "type1",
            vec![
                build_element("inner1", "type2"),
                build_element("inner2", "uint64_le"),
            ],
        );
        let t2 = build_struct("type2", vec![build_element("inner3", "int8")]);
        let schema = Schema {
            structs: vec![t1, t2],
        };
        type_check_schema(schema)?;
        Ok(())
    }

    #[test]
    fn undefined_type() {
        let t1 = build_struct(
            "type1",
            vec![
                build_element("inner1", "type2"),
                build_element("inner2", "uint64_le"),
            ],
        );
        let schema = Schema { structs: vec![t1] };
        let res = type_check_schema(schema);
        assert_eq!(res, Err(CartaError::new_unknown_type(0, "type2".to_string())));
    }

    #[test]
    fn type_loop() {
        let t1 = build_struct(
            "type1",
            vec![
                build_element("inner1", "type2"),
                build_element("inner2", "uint64_le"),
            ],
        );
        let t2 = build_struct(
            "type2",
            vec![
                build_element("inner3", "type1"),
                build_element("inner4", "int8"),
            ],
        );
        let schema = Schema {
            structs: vec![t1, t2],
        };
        let res = type_check_schema(schema);
        if let Err(CartaError {line_no: 0, code: CartaErrorCode::RecursiveTypes(data)}) = res {
            compare_vec_unordered(data, vec!["type1".to_string(), "type2".to_string()])
        } else {
            panic!("Unexpected value: {:?}", res);
        }
    }

    #[test]
    fn many_types() -> Result<(), CartaError> {
        let t1 = build_struct(
            "type1",
            vec![
                build_element("inner1", "type2"),
                build_element("inner2", "type3"),
            ],
        );
        let t2 = build_struct("type2", vec![build_element("inner3", "type4")]);
        let t3 = build_struct(
            "type3",
            vec![
                build_element("inner1", "type5"),
                build_element("inner2", "type6"),
            ],
        );
        let t4 = build_struct("type4", vec![build_element("inner3", "type5")]);
        let t5 = build_struct("type5", vec![build_element("inner3", "type6")]);
        let t6 = build_struct(
            "type6",
            vec![
                build_element("inner1", "int8"),
                build_element("inner2", "f32_be"),
            ],
        );
        let schema = Schema {
            structs: vec![t1, t2, t3, t4, t5, t6],
        };
        type_check_schema(schema)?;
        Ok(())
    }

    #[test]
    fn type_loop_long_chain() {
        let t1 = build_struct(
            "type1",
            vec![
                build_element("inner1", "type2"),
                build_element("inner2", "type3"),
            ],
        );
        let t2 = build_struct(
            "type2",
            vec![
                build_element("inner3", "type3"),
                build_element("inner4", "int8"),
            ],
        );
        let t3 = build_struct(
            "type3",
            vec![
                build_element("inner3", "type4"),
                build_element("inner4", "type5"),
            ],
        );
        let t4 = build_struct(
            "type4",
            vec![
                build_element("inner3", "type7"),
                build_element("inner4", "int8"),
            ],
        );
        let t5 = build_struct(
            "type5",
            vec![
                build_element("inner3", "type6"),
                build_element("inner4", "uint8"),
            ],
        );
        let t6 = build_struct(
            "type6",
            vec![
                build_element("inner3", "f64_le"),
                build_element("inner4", "int64_be"),
            ],
        );
        let t7 = build_struct(
            "type7",
            vec![
                build_element("inner3", "f64_be"),
                build_element("inner4", "int64_le"),
                build_element("inner3", "uint32_be"),
                build_element("inner4", "type2"),
            ],
        );
        let schema = Schema {
            structs: vec![t1, t2, t3, t4, t5, t6, t7],
        };
        let res = type_check_schema(schema);
        if let Err(CartaError {line_no: 0, code: CartaErrorCode::RecursiveTypes(data)}) = res {
            compare_vec_unordered(
                data,
                vec![
                    "type2".to_string(),
                    "type3".to_string(),
                    "type7".to_string(),
                    "type1".to_string(),
                    "type4".to_string(),
                ],
            )
        } else {
            panic!("Unexpected value: {:?}", res);
        }
    }

    #[test]
    fn duplicate_types() {
        let t1 = build_struct(
            "type1",
            vec![
                build_element("inner1", "uint8"),
                build_element("inner2", "uint64_le"),
            ],
        );
        let t2 = build_struct("type1", vec![build_element("inner3", "type1")]);
        let schema = Schema {
            structs: vec![t1, t2],
        };
        let res = type_check_schema(schema);
        assert_eq!(res, Err(CartaError::new_duplicate_type(0, "type1".to_string())));
    }

    #[test]
    fn recursive_type() {
        let t1 = build_struct(
            "type1",
            vec![
                build_element("inner1", "type1"),
                build_element("inner2", "uint64_le"),
            ],
        );
        let schema = Schema { structs: vec![t1] };
        let res = type_check_schema(schema);
        assert_eq!(
            res,
            Err(CartaError::new_recursive_types(0, vec!["type1".to_string()]))
        );
    }

    #[test]
    fn element_bad_typename() {
        let t1 = build_struct(
            "type1",
            vec![
                build_element("inner1", "bad_type"),
                build_element("inner2", "uint64_le"),
            ],
        );
        let schema = Schema { structs: vec![t1] };
        let res = type_check_schema(schema);
        assert_eq!(res, Err(CartaError::new_unknown_type(0, "bad_type".to_string())));
    }
}
