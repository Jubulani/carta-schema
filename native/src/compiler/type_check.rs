use std::collections::{HashMap, HashSet};

use crate::compiler::builtin_types;
use crate::compiler::parser::{Nugget, NuggetStructDefn, NuggetTypeRef, Schema};

pub struct TSchema {
    pub nuggets: Vec<Nugget>,
    pub types: HashMap<String, NuggetStructDefn>,
}

pub fn type_check_schema(schema: Schema) -> TSchema {
    let Schema { nuggets, types } = schema;

    let types = check_types(types);

    check_nuggets(&nuggets, &types);

    TSchema { nuggets, types }
}

fn build_types_map(types: Vec<NuggetStructDefn>) -> HashMap<String, NuggetStructDefn> {
    let mut types_map: HashMap<String, NuggetStructDefn> = HashMap::new();

    for kind in types.into_iter() {
        if types_map.contains_key::<str>(&kind.name) {
            panic!("Duplicate definition of type: {}", &kind.name);
        }
        types_map.insert(kind.name.clone(), kind);
        //ret.push(kind);
    }

    types_map
}

fn check_all_types_defined(types_map: &HashMap<String, NuggetStructDefn>) {
    // All types are now stored in types_map.  We can now go over all members of all types, and
    // check that they've all been defined.
    for kind in types_map.values() {
        for member in &kind.members {
            let typename = match &member.kind {
                NuggetTypeRef::TypeName(s) => s,
            };
            if !builtin_types::is_builtin_type(&typename)
                && types_map.get::<str>(&typename).is_none()
            {
                panic!("Undefined type: {}", typename);
            }
        }
    }
}

/// Check that there are no types that recursively depend on themselves.
fn check_types_no_loops(types_map: &HashMap<String, NuggetStructDefn>) {
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
        for member in &kind.members {
            let typename = match &member.kind {
                NuggetTypeRef::TypeName(s) => s,
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

        match all_builtin {
            true => {
                types_resolved.insert(&kind.name);
                types_stack.push(&kind.name);
            }
            false => {}
        }
    }

    // Go over the stack of resolved types.  Use the dependant_types map to check if the types
    // the depend on the resolved type can be marked as resolved.
    while let Some(kind_name) = types_stack.pop() {
        if let Some(parents) = dependant_types.get::<str>(&kind_name) {
            for parent in parents.iter() {
                let parent = match types_map.get::<str>(parent) {
                    Some(p) => p,
                    None => panic!("Unresolved type: {:?}", parent),
                };

                let mut all_resolved = true;
                for member in &parent.members {
                    let typename = match &member.kind {
                        NuggetTypeRef::TypeName(s) => s,
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
            recursive_types.push(&kind.name);
        }
    }
    if recursive_types.len() > 0 {
        panic!("Recursive types detected with types: {:?}", recursive_types);
    }
}

fn check_types(types: Vec<NuggetStructDefn>) -> HashMap<String, NuggetStructDefn> {
    let types_map = build_types_map(types);
    check_all_types_defined(&types_map);
    check_types_no_loops(&types_map);

    types_map
}

// Check that all nugget types have been defined
fn check_nuggets(nuggets: &Vec<Nugget>, types_map: &HashMap<String, NuggetStructDefn>) {
    for nugget in nuggets.iter() {
        let typename = match &nugget.kind {
            NuggetTypeRef::TypeName(s) => s,
        };
        if !types_map.contains_key::<str>(&typename) {
            panic!("Unknown type name: {}", &typename);
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    fn build_nugget(name: &str, typename: &str) -> Nugget {
        Nugget {
            name: name.to_string(),
            kind: NuggetTypeRef::TypeName(typename.to_string()),
        }
    }

    fn build_type(name: &str, nuggets: Vec<Nugget>) -> NuggetStructDefn {
        NuggetStructDefn {
            name: name.to_string(),
            members: nuggets,
        }
    }

    #[test]
    fn test_basic() {
        let tnugget1 = build_nugget("inner1", "uint16_le");
        let schema = Schema {
            nuggets: vec![build_nugget("new_name", "type1")],
            types: vec![build_type("type1", vec![tnugget1])],
        };
        type_check_schema(schema);
    }

    #[test]
    fn test_multi() {
        let t1 = build_type(
            "type1",
            vec![
                build_nugget("inner1", "type2"),
                build_nugget("inner2", "uint64_le"),
            ],
        );
        let t2 = build_type("type2", vec![build_nugget("inner3", "int8")]);
        let schema = Schema {
            nuggets: vec![build_nugget("name1", "type1")],
            types: vec![t1, t2],
        };
        type_check_schema(schema);
    }

    #[test]
    #[should_panic(expected = "Undefined type: type2")]
    fn test_undefined_type() {
        let t1 = build_type(
            "type1",
            vec![
                build_nugget("inner1", "type2"),
                build_nugget("inner2", "uint64_le"),
            ],
        );
        let schema = Schema {
            nuggets: vec![build_nugget("name1", "type1")],
            types: vec![t1],
        };
        type_check_schema(schema);
    }

    #[test]
    #[should_panic(expected = "Recursive types detected")]
    fn test_type_loop() {
        let t1 = build_type(
            "type1",
            vec![
                build_nugget("inner1", "type2"),
                build_nugget("inner2", "uint64_le"),
            ],
        );
        let t2 = build_type(
            "type2",
            vec![
                build_nugget("inner3", "type1"),
                build_nugget("inner4", "int8"),
            ],
        );
        let schema = Schema {
            nuggets: vec![build_nugget("name1", "type1")],
            types: vec![t1, t2],
        };
        type_check_schema(schema);
    }

    #[test]
    fn test_many_types() {
        let t1 = build_type(
            "type1",
            vec![
                build_nugget("inner1", "type2"),
                build_nugget("inner2", "type3"),
            ],
        );
        let t2 = build_type("type2", vec![build_nugget("inner3", "type4")]);
        let t3 = build_type(
            "type3",
            vec![
                build_nugget("inner1", "type5"),
                build_nugget("inner2", "type6"),
            ],
        );
        let t4 = build_type("type4", vec![build_nugget("inner3", "type5")]);
        let t5 = build_type("type5", vec![build_nugget("inner3", "type6")]);
        let t6 = build_type(
            "type6",
            vec![
                build_nugget("inner1", "int8"),
                build_nugget("inner2", "f32_be"),
            ],
        );
        let schema = Schema {
            nuggets: vec![build_nugget("name1", "type1")],
            types: vec![t1, t2, t3, t4, t5, t6],
        };
        type_check_schema(schema);
    }

    #[test]
    #[should_panic(expected = "Recursive types detected")]
    fn test_type_loop_long_chain() {
        let t1 = build_type(
            "type1",
            vec![
                build_nugget("inner1", "type2"),
                build_nugget("inner2", "type3"),
            ],
        );
        let t2 = build_type(
            "type2",
            vec![
                build_nugget("inner3", "type3"),
                build_nugget("inner4", "int8"),
            ],
        );
        let t3 = build_type(
            "type3",
            vec![
                build_nugget("inner3", "type4"),
                build_nugget("inner4", "type5"),
            ],
        );
        let t4 = build_type(
            "type4",
            vec![
                build_nugget("inner3", "type7"),
                build_nugget("inner4", "int8"),
            ],
        );
        let t5 = build_type(
            "type5",
            vec![
                build_nugget("inner3", "type6"),
                build_nugget("inner4", "uint8"),
            ],
        );
        let t6 = build_type(
            "type6",
            vec![
                build_nugget("inner3", "f64_le"),
                build_nugget("inner4", "int64_be"),
            ],
        );
        let t7 = build_type(
            "type7",
            vec![
                build_nugget("inner3", "f64_be"),
                build_nugget("inner4", "int64_le"),
                build_nugget("inner3", "uint32_be"),
                build_nugget("inner4", "type2"),
            ],
        );
        let schema = Schema {
            nuggets: vec![],
            types: vec![t1, t2, t3, t4, t5, t6, t7],
        };
        type_check_schema(schema);
    }

    #[test]
    #[should_panic(expected = "Duplicate definition of type: type1")]
    fn test_duplicate_types() {
        let t1 = build_type(
            "type1",
            vec![
                build_nugget("inner1", "uint8"),
                build_nugget("inner2", "uint64_le"),
            ],
        );
        let t2 = build_type("type1", vec![build_nugget("inner3", "type1")]);
        let schema = Schema {
            nuggets: vec![build_nugget("name1", "type1")],
            types: vec![t1, t2],
        };
        type_check_schema(schema);
    }

    #[test]
    #[should_panic(expected = "Recursive types detected with types: [\"type1\"]")]
    fn test_recursive_type() {
        let t1 = build_type(
            "type1",
            vec![
                build_nugget("inner1", "type1"),
                build_nugget("inner2", "uint64_le"),
            ],
        );
        let schema = Schema {
            nuggets: vec![build_nugget("name1", "type1")],
            types: vec![t1],
        };
        type_check_schema(schema);
    }

    #[test]
    #[should_panic(expected = "Unknown type name: bad_type")]
    fn test_nugget_bad_typename() {
        let t1 = build_type(
            "type1",
            vec![
                build_nugget("inner1", "uint8"),
                build_nugget("inner2", "uint64_le"),
            ],
        );
        let schema = Schema {
            nuggets: vec![build_nugget("name1", "bad_type")],
            types: vec![t1],
        };
        type_check_schema(schema);
    }
}
