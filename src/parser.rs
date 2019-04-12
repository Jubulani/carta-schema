use crate::error::CartaError;
use crate::tokeniser::{Token, TokenType, Tokeniser};

#[derive(PartialEq, Debug)]
pub struct Schema {
    pub structs: Vec<StructDefn>,
}

impl Schema {
    fn add_struct(&mut self, s: StructDefn) {
        self.structs.push(s);
    }
}

#[derive(PartialEq, Debug)]
pub struct Element {
    pub name: String,
    pub kind: ElementTypeRef,
}

#[derive(PartialEq, Debug)]
pub enum ElementTypeRef {
    TypeName(String),
}

#[derive(PartialEq, Debug)]
pub struct StructDefn {
    pub name: String,
    pub elements: Vec<Element>,
}

trait CompilerState {
    fn new_token(
        self: Box<Self>,
        t: Token,
        schema: &mut Schema,
    ) -> Result<Box<dyn CompilerState>, CartaError>;
}

struct EmptyState;

impl CompilerState for EmptyState {
    fn new_token(
        self: Box<Self>,
        t: Token,
        _: &mut Schema,
    ) -> Result<Box<dyn CompilerState>, CartaError> {
        if let Some(s) = new_state(t)? {
            return Ok(s);
        }
        Ok(self)
    }
}

struct StructState {
    state: StructSubState,
    name: Option<String>,
    complete_children: Vec<Element>,
    new_child_name: Option<String>,
}

#[derive(PartialEq)]
enum StructSubState {
    Begin,
    Name,
    OpenBrace,
    ChildName,
    ChildTypeOf,
    ChildKind,
}

impl StructState {
    fn new() -> StructState {
        StructState {
            state: StructSubState::Begin,
            name: None,
            complete_children: Vec::new(),
            new_child_name: None,
        }
    }

    fn add_complete_struct(self: Box<Self>, schema: &mut Schema) {
        let defn = StructDefn {
            name: self.name.unwrap(),
            elements: self.complete_children,
        };
        schema.add_struct(defn);
    }
}

impl CompilerState for StructState {
    fn new_token(
        mut self: Box<Self>,
        t: Token,
        schema: &mut Schema,
    ) -> Result<Box<dyn CompilerState>, CartaError> {
        // New lines are ignored in struct definitions
        if t.kind == TokenType::NewLine {
            return Ok(self);
        }

        match self.state {
            StructSubState::Begin => {
                if t.kind != TokenType::Word {
                    return Err(CartaError::ParseError("<name>".to_string(), t.value));
                }
                self.name = Some(t.value);
                self.state = StructSubState::Name;
            }
            StructSubState::Name => {
                // Next state must be OpenBrace
                if t.kind != TokenType::OpenBrace {
                    return Err(CartaError::ParseError("{".to_string(), t.value));
                }
                self.state = StructSubState::OpenBrace;
            }
            StructSubState::OpenBrace => match t.kind {
                TokenType::CloseBrace => {
                    // Struct is complete, maybe with child elements
                    self.add_complete_struct(schema);
                    return Ok(Box::new(EmptyState {}));
                }
                TokenType::Word => {
                    self.new_child_name = Some(t.value);
                    self.state = StructSubState::ChildName;
                }
                _ => return Err(CartaError::ParseError("}".to_string(), t.value)),
            },
            StructSubState::ChildName => {
                // Next state must be TypeOf
                if t.kind != TokenType::TypeOf {
                    return Err(CartaError::ParseError(":".to_string(), t.value));
                }
                self.state = StructSubState::ChildTypeOf;
            }
            StructSubState::ChildTypeOf => {
                // Next state must be a typename
                if t.kind != TokenType::Word {
                    return Err(CartaError::ParseError("<typename>".to_string(), t.value));
                }
                let kind = t.value;
                self.complete_children.push(Element {
                    name: self.new_child_name.take().unwrap(),
                    kind: ElementTypeRef::TypeName(kind),
                });
                self.state = StructSubState::ChildKind;
            }
            StructSubState::ChildKind => {
                match t.kind {
                    // Next state may be a comma
                    TokenType::Comma => self.state = StructSubState::OpenBrace,
                    // Or a close brace if there is no comma after the last element
                    TokenType::CloseBrace => {
                        self.add_complete_struct(schema);
                        return Ok(Box::new(EmptyState {}));
                    }
                    _ => return Err(CartaError::ParseError(",".to_string(), t.value)),
                }
            }
        }

        Ok(self)
    }
}

fn new_state(t: Token) -> Result<Option<Box<dyn CompilerState>>, CartaError> {
    if t.kind == TokenType::Word {
        // Match against language keywords
        return match &*t.value {
            "struct" => Ok(Some(Box::new(StructState::new()))),
            _ => Err(CartaError::ParseError("<keyword>".to_string(), t.value)),
        };
    } else if t.kind == TokenType::NewLine {
        // Empty newline - nothing to parse
        return Ok(None);
    }

    return Err(CartaError::ParseError("<keyword>".to_string(), t.value));
}

pub fn compile_schema(tokeniser: Tokeniser) -> Result<Schema, CartaError> {
    let mut schema = Schema {
        structs: Vec::new(),
    };
    let mut state: Box<dyn CompilerState> = Box::new(EmptyState {});
    for token in tokeniser.into_iter() {
        state = state.new_token(token, &mut schema)?;
    }

    // TODO: Handle incomplete input still remaining.

    Ok(schema)
}

#[cfg(test)]
mod test {
    use super::*;

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

    #[test]
    fn test_basic_builtin() -> Result<(), CartaError> {
        let tokeniser = Tokeniser::new("struct s {new_name: uint64_le}")?;
        let schema = compile_schema(tokeniser)?;
        let mut iter = schema.structs.iter();
        assert_eq!(
            iter.next(),
            Some(&build_struct(
                "s",
                vec![build_element("new_name", "uint64_le")]
            ))
        );
        assert_eq!(iter.next(), None);
        Ok(())
    }

    #[test]
    fn test_empty_input() -> Result<(), CartaError> {
        let tokeniser = Tokeniser::new("")?;
        let schema = compile_schema(tokeniser)?;
        let mut iter = schema.structs.iter();
        assert_eq!(iter.next(), None);
        Ok(())
    }

    #[test]
    fn test_whitespace_input() -> Result<(), CartaError> {
        let tokeniser = Tokeniser::new("\n  \t\n\n")?;
        let schema = compile_schema(tokeniser)?;
        let mut iter = schema.structs.iter();
        assert_eq!(iter.next(), None);
        Ok(())
    }

    #[test]
    fn test_multiple_builtins() -> Result<(), CartaError> {
        let tokeniser = Tokeniser::new(
            "struct s {
                name1: int8,
                name2: uint64_be,
                name3: f64_le,
            }",
        )?;
        let schema = compile_schema(tokeniser)?;
        let mut iter = schema.structs.iter();
        assert_eq!(
            iter.next(),
            Some(&build_struct(
                "s",
                vec![
                    build_element("name1", "int8"),
                    build_element("name2", "uint64_be"),
                    build_element("name3", "f64_le"),
                ]
            ))
        );
        assert_eq!(iter.next(), None);
        Ok(())
    }

    #[test]
    fn test_struct_syntax_errors() -> Result<(), CartaError> {
        let tokeniser = Tokeniser::new(
            "struct new_type {
                inner_val1: int8,
                inner_val2, int8,
            }",
        )?;
        let ret = compile_schema(tokeniser);
        assert_eq!(
            ret,
            Err(CartaError::ParseError(":".to_string(), ",".to_string()))
        );

        let tokeniser = Tokeniser::new(
            "struct new_type {
                inner_val1: int8,
                inner_val2: int8,:
            }",
        )?;
        let ret = compile_schema(tokeniser);
        assert_eq!(
            ret,
            Err(CartaError::ParseError("}".to_string(), ":".to_string()))
        );

        let tokeniser = Tokeniser::new(
            "struct {
                inner_val1: int8,
                inner_val2: int8,:
            }",
        )?;
        let ret = compile_schema(tokeniser);
        assert_eq!(
            ret,
            Err(CartaError::ParseError(
                "<name>".to_string(),
                "{".to_string()
            ))
        );

        let tokeniser = Tokeniser::new(
            "struct new_type
                inner_val1: int8,
                inner_val2: int8,:
            }",
        )?;
        let ret = compile_schema(tokeniser);
        assert_eq!(
            ret,
            Err(CartaError::ParseError(
                "{".to_string(),
                "inner_val1".to_string()
            ))
        );

        let tokeniser = Tokeniser::new(
            "struct new_type {
                inner_val1: int8,
                inner_val2: int8,:
            }",
        )?;
        let ret = compile_schema(tokeniser);
        assert_eq!(
            ret,
            Err(CartaError::ParseError("}".to_string(), ":".to_string()))
        );

        let tokeniser = Tokeniser::new(
            "struct new_type {
                inner_val1: ,
                inner_val2: int8,
            }",
        )?;
        let ret = compile_schema(tokeniser);
        assert_eq!(
            ret,
            Err(CartaError::ParseError(
                "<typename>".to_string(),
                ",".to_string()
            ))
        );

        let tokeniser = Tokeniser::new(
            "struct new_type {
                inner_val1: int8
                inner_val2: int8,
            }",
        )?;
        let ret = compile_schema(tokeniser);
        assert_eq!(
            ret,
            Err(CartaError::ParseError(
                ",".to_string(),
                "inner_val2".to_string()
            ))
        );
        Ok(())
    }
}
