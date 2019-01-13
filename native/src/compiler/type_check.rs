use std::collections::{HashMap, HashSet};

use crate::compiler::parser::{ILNugget, NuggetStructDefn, NuggetTypeRef, Schema};
use crate::compiler::types;

pub struct TSchema {
    nuggets: Vec<TNugget>,
    types: Vec<TNuggetStructDefn>,
}

struct TNugget {
    name: String,
    kind: TNuggetType,
}

#[derive(PartialEq, Debug)]
enum TNuggetType {
    TNSimpleType(String),
    TNCompountType { struct_handle: usize },
}

struct TNuggetStructDefn {
    name: String,
    members: Vec<TNugget>,
}

pub fn type_check_schema(schema: Schema) {
    let tSchema = TSchema {
        nuggets: Vec::new(),
        types: Vec::new(),
    };

    let Schema { nuggets, types } = schema;

    let types = check_types(types);

    check_nuggets(nuggets);
}

// To do here:
// - Check all types are defined
// - Check no loops
// - Resolve all type references
fn check_types(types: Vec<NuggetStructDefn>) -> Vec<TNuggetStructDefn> {
    let ret: Vec<TNuggetStructDefn> = Vec::new();

    // Add all types to a set.  This allows us to check they have all been defined, and resolve
    // loopups
    let mut types_set: HashMap<&str, &NuggetStructDefn> = HashMap::new();

    for kind in types.iter() {
        types_set.insert(&kind.name, &kind);
        //ret.push(kind);
    }
    println!("All types: {:?}", types_set);

    // All types are now stored in types_set.  We can now go over all members of all types, and
    // check that they've been defined.
    for kind in types.iter() {
        for member in &kind.members {
            let typename = match &member.kind {
                NuggetTypeRef::TypeName(s) => s,
            };
            if !types::is_builtin_type(&typename) && types_set.get::<str>(&typename).is_none() {
                panic!("Undefined type: {}", typename);
            }
        }
    }

    let mut types_resolved: HashSet<&str> = HashSet::new();
    let mut dependant_types: HashMap<&str, Vec<&str>> = HashMap::new();
    let mut types_stack: Vec<&str> = Vec::new();

    // Check for loops
    for kind in types.iter() {
        println!("Check type: {:?}", &kind.name);
        let mut all_builtin = true;
        for member in &kind.members {
            let typename = match &member.kind {
                NuggetTypeRef::TypeName(s) => s,
            };
            if !types::is_builtin_type(&typename) && types_resolved.get::<str>(&typename).is_none()
            {
                println!("Member not found: {:?}", &typename);
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
        println!("Is all builtin: {:?}", all_builtin);
        println!("Dependant types: {:?}", dependant_types);

        match all_builtin {
            true => {
                types_resolved.insert(&kind.name);
                types_stack.push(&kind.name);
                /*if let Some(parents) = dependant_types.get::<str>(&kind.name) {
                    println!("Have parents: {:?}", parents);
                    for parent in parents.iter() {
                        let parent = match types_set.get(parent) {
                            Some(p) => p,
                            None => panic!("Unresolved type: {:?}", parent),
                        };
                        println!("Check parent: {:?}", parent);
                        for member in &parent.members {
                            println!("Check member: {:?}", member);
                        }
                    }
                }*/
            }
            false => {}
        }
    }
    println!("types_stack: {:?}", types_stack);

    while let Some(kind_name) = types_stack.pop() {
        println!("Pop kind: {:?}", &kind_name);
        if let Some(parents) = dependant_types.get::<str>(&kind_name) {
            println!("Have parents: {:?}", parents);
            for parent in parents.iter() {
                let parent = match types_set.get(parent) {
                    Some(p) => p,
                    None => panic!("Unresolved type: {:?}", parent),
                };
                println!("Check parent: {:?}", parent);
                let mut all_resolved = true;
                for member in &parent.members {
                    println!("Check member: {:?}", member);
                    let typename = match &member.kind {
                        NuggetTypeRef::TypeName(s) => s,
                    };
                    if !types::is_builtin_type(&typename)
                        && types_resolved.get::<str>(&typename).is_none()
                    {
                        println!("Unresolved: {:?}", &typename);
                        all_resolved = false;
                    }
                }
                if all_resolved {
                    println!("Push type: {:?}", &parent.name);
                    types_resolved.insert(&parent.name);
                    types_stack.push(&parent.name);
                }
            }
        }
    }

    // If any types remain that aren't listed in types_resolved, then we must have a loop
    let mut recursive_types = Vec::new();
    for kind in types.iter() {
        if types_resolved.get::<str>(&kind.name).is_none() {
            recursive_types.push(&kind.name);
        }
    }
    if recursive_types.len() > 0 {
        panic!("Recursive types detected with types: {:?}", recursive_types);
    }

    ret
}

fn check_nuggets(nuggets: Vec<ILNugget>) {
    for nugget in nuggets.iter() {}
}

#[cfg(test)]
mod test {
    use super::*;

    fn build_nugget(name: &str, typename: &str) -> ILNugget {
        ILNugget {
            name: name.to_string(),
            kind: NuggetTypeRef::TypeName(typename.to_string()),
        }
    }

    fn build_type(name: &str, nuggets: Vec<ILNugget>) -> NuggetStructDefn {
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
}
