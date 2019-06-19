use crate::builtin_types;
use crate::builtin_types::BuiltinTypeClass;
use crate::error::CartaError;
use crate::parser::{ArrayDefn, ArrayLen, ElementTypeRef, StructDefn};
use crate::type_check::TSchema;

pub fn check_schema(schema: &TSchema) -> Result<(), CartaError> {
    check_root_element(schema)?;
    check_array_lengths(schema)?;
    Ok(())
}

fn check_root_element(schema: &TSchema) -> Result<(), CartaError> {
    return if !schema.types.contains_key("root") {
        Err(CartaError::new_missing_root_element(0))
    } else {
        Ok(())
    };
}

fn check_array_lengths(schema: &TSchema) -> Result<(), CartaError> {
    for (_name, struct_defn) in &schema.types {
        for i in 0..struct_defn.elements.len() {
            if let ElementTypeRef::ArrayElem(arr) = &struct_defn.elements[i].kind {
                check_array_elem(struct_defn, arr, i)?;
            }
        }
    }

    Ok(())
}

fn check_array_elem(
    struct_defn: &StructDefn,
    arr: &ArrayDefn,
    arr_idx: usize,
) -> Result<(), CartaError> {
    match &arr.length {
        // Nothing to check
        ArrayLen::Static(_) => Ok(()),
        // Check that the element we reference is a builtin integer type
        ArrayLen::Identifier(id) => {
            for j in 0..arr_idx {
                if struct_defn.elements[j].name == *id {
                    // Check that this element is a builtin type that is an integer type
                    if let ElementTypeRef::TypeName(typename) = &struct_defn.elements[j].kind {
                        if builtin_types::is_type_class(typename, BuiltinTypeClass::Integer) {
                            return Ok(());
                        } else {
                            return Err(CartaError::new_bad_array_len_type(0, id));
                        }
                    } else {
                        return Err(CartaError::new_bad_array_len_type(0, id));
                    }
                }
            }
            Err(CartaError::new_bad_array_len(0, id))
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::parser;
    use crate::parser::{ArrayDefn, Element, StructDefn};
    use crate::tokeniser;
    use crate::type_check;

    use std::collections::HashMap;

    fn build_schema_with_elem(name: String) -> TSchema {
        let mut types = HashMap::new();
        types.insert(
            name.clone(),
            StructDefn {
                name,
                elements: Vec::new(),
            },
        );
        TSchema { types }
    }

    #[test]
    fn basic_ok() -> Result<(), CartaError> {
        let schema = build_schema_with_elem("root".to_string());
        check_schema(&schema)?;
        Ok(())
    }

    #[test]
    fn no_root() {
        let schema = build_schema_with_elem("notroot".to_string());
        let res = check_schema(&schema);
        assert_eq!(res, Err(CartaError::new_missing_root_element(0)));
    }

    #[test]
    fn bad_arr_len() {
        let mut schema = build_schema_with_elem("root".to_string());
        schema.types.insert(
            "foo".to_string(),
            StructDefn {
                name: "foo".to_string(),
                elements: vec![Element {
                    name: "foo".to_string(),
                    kind: ElementTypeRef::ArrayElem(ArrayDefn {
                        kind: "int8".to_string(),
                        length: ArrayLen::Identifier("unknown".to_string()),
                    }),
                }],
            },
        );
        let res = check_schema(&schema);
        assert_eq!(res, Err(CartaError::new_bad_array_len(0, "unknown")));
    }

    #[test]
    fn arr_len_not_builtin() {
        let data =
            "struct root {var1: Version, var2: [uint16_be; var1]} struct Version {major: f64_le}";
        let tokeniser = tokeniser::Tokeniser::new(&data).unwrap();
        let schema = parser::compile_schema(tokeniser).unwrap();
        let tschema = type_check::type_check_schema(schema).unwrap();
        let res = check_schema(&tschema);
        assert_eq!(res, Err(CartaError::new_bad_array_len_type(0, "var1")));
    }

    #[test]
    fn arr_len_not_integer() {
        let data = "struct root {var1: f32_be, var2: [uint16_le; var1]}";
        let tokeniser = tokeniser::Tokeniser::new(&data).unwrap();
        let schema = parser::compile_schema(tokeniser).unwrap();
        let tschema = type_check::type_check_schema(schema).unwrap();
        let res = check_schema(&tschema);
        assert_eq!(res, Err(CartaError::new_bad_array_len_type(0, "var1")));
    }
}
