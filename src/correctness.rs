use crate::error::CartaError;
use crate::type_check::TSchema;

pub fn check_schema(schema: &TSchema) -> Result<(), CartaError> {
    check_root_element(schema)?;
    Ok(())
}

fn check_root_element(schema: &TSchema) -> Result<(), CartaError> {
    return if !schema.types.contains_key("root") {
        Err(CartaError::MissingRootElement())
    } else {
        Ok(())
    };
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::parser::StructDefn;
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
    fn test_ok() -> Result<(), CartaError> {
        let schema = build_schema_with_elem("root".to_string());
        check_schema(&schema)?;
        Ok(())
    }

    #[test]
    fn test_no_root() {
        let schema = build_schema_with_elem("notroot".to_string());
        let res = check_schema(&schema);
        assert_eq!(res, Err(CartaError::MissingRootElement()));
    }
}
