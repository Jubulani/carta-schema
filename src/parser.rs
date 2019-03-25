use crate::error::CartaError;
use crate::tokeniser::{Token, TokenType, Tokeniser};

#[derive(PartialEq, Debug)]
pub struct Schema {
    pub nuggets: Vec<Nugget>,
    pub types: Vec<NuggetStructDefn>,
}

impl Schema {
    fn add_nugget(&mut self, n: Nugget) {
        self.nuggets.push(n);
    }

    fn add_struct(&mut self, s: NuggetStructDefn) {
        self.types.push(s);
    }
}

#[derive(PartialEq, Debug)]
pub struct Nugget {
    pub name: String,
    pub kind: NuggetTypeRef,
}

#[derive(PartialEq, Debug)]
pub enum NuggetTypeRef {
    TypeName(String),
}

#[derive(PartialEq, Debug)]
pub struct NuggetStructDefn {
    pub name: String,
    pub members: Vec<Nugget>,
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

struct NewNuggetState {
    state: NewNuggetSubState,
    name: String,
    kind: Option<NuggetTypeRef>,
}

#[derive(PartialEq)]
enum NewNuggetSubState {
    Name,
    TypeOf,
    Kind,
}

impl NewNuggetState {
    fn new(t: Token) -> NewNuggetState {
        NewNuggetState {
            state: NewNuggetSubState::Name,
            name: t.value,
            kind: None,
        }
    }
}

impl CompilerState for NewNuggetState {
    fn new_token(
        mut self: Box<Self>,
        t: Token,
        schema: &mut Schema,
    ) -> Result<Box<dyn CompilerState>, CartaError> {
        match self.state {
            NewNuggetSubState::Name => {
                // Next state must be TypeOf
                if t.kind != TokenType::TypeOf {
                    return Err(CartaError::ParseError(":".to_string(), t.value));
                }
                self.state = NewNuggetSubState::TypeOf;
            }
            NewNuggetSubState::TypeOf => {
                // Next state must be the type name
                self.kind = Some(NuggetTypeRef::TypeName(t.value));
                self.state = NewNuggetSubState::Kind;
            }
            NewNuggetSubState::Kind => {
                // Next state must be a newline
                if t.kind != TokenType::NewLine {
                    return Err(CartaError::ParseError("<newline>".to_string(), t.value));
                }

                if let Some(kind) = self.kind {
                    let nugget = Nugget {
                        name: self.name,
                        kind,
                    };
                    schema.add_nugget(nugget);
                    return Ok(Box::new(EmptyState));
                } else {
                    panic!("Kind not available");
                }
            }
        }

        Ok(self)
    }
}

struct StructState {
    state: StructSubState,
    name: Option<String>,
    complete_children: Vec<Nugget>,
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
                    // Struct is complete, maybe with child types
                    let defn = NuggetStructDefn {
                        name: self.name.unwrap(),
                        members: self.complete_children,
                    };
                    schema.add_struct(defn);
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
                self.complete_children.push(Nugget {
                    name: self.new_child_name.take().unwrap(),
                    kind: NuggetTypeRef::TypeName(kind),
                });
                self.state = StructSubState::ChildKind;
            }
            StructSubState::ChildKind => {
                // Next state must be Comma
                if t.kind != TokenType::Comma {
                    return Err(CartaError::ParseError(",".to_string(), t.value));
                }
                self.state = StructSubState::OpenBrace;
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

            // If not a keyword, must be a new nugget name
            _ => Ok(Some(Box::new(NewNuggetState::new(t)))),
        };
    } else if t.kind == TokenType::NewLine {
        // Empty newline - nothing to parse
        return Ok(None);
    }

    return Err(CartaError::ParseError("<name>".to_string(), t.value));
}

pub fn compile_schema(tokeniser: Tokeniser) -> Result<Schema, CartaError> {
    let mut schema = Schema {
        nuggets: Vec::new(),
        types: Vec::new(),
    };
    let mut state: Box<dyn CompilerState> = Box::new(EmptyState {});
    for token in tokeniser.into_iter() {
        state = state.new_token(token, &mut schema)?;
    }

    // Add a final newline, in case one doesn't exist in the input
    // This will flush any remaining (valid) nuggets
    state.new_token(
        Token::new(TokenType::NewLine, "\n".to_string()),
        &mut schema,
    )?;

    // TODO: Handle incomplete nuggets still remaining.

    Ok(schema)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_basic_builtin() -> Result<(), CartaError> {
        let tokeniser = Tokeniser::new("new_name: uint64_le")?;
        let schema = compile_schema(tokeniser)?;
        let mut iter = schema.nuggets.iter();
        assert_eq!(
            iter.next(),
            Some(&Nugget {
                name: "new_name".to_string(),
                kind: NuggetTypeRef::TypeName("uint64_le".to_string()),
            })
        );
        assert_eq!(iter.next(), None);
        Ok(())
    }

    #[test]
    fn test_empty_input() -> Result<(), CartaError> {
        let tokeniser = Tokeniser::new("")?;
        let schema = compile_schema(tokeniser)?;
        let mut iter = schema.nuggets.iter();
        assert_eq!(iter.next(), None);
        Ok(())
    }

    #[test]
    fn test_whitespace_input() -> Result<(), CartaError> {
        let tokeniser = Tokeniser::new("\n  \t\n\n")?;
        let schema = compile_schema(tokeniser)?;
        let mut iter = schema.nuggets.iter();
        assert_eq!(iter.next(), None);
        Ok(())
    }

    #[test]
    fn test_multiple_builtins() -> Result<(), CartaError> {
        let tokeniser = Tokeniser::new(
            "name1: int8
            name2: uint64_be
            name3: f64_le
        ",
        )?;
        let schema = compile_schema(tokeniser)?;
        let mut iter = schema.nuggets.iter();
        assert_eq!(
            iter.next(),
            Some(&Nugget {
                name: "name1".to_string(),
                kind: NuggetTypeRef::TypeName("int8".to_string()),
            })
        );
        assert_eq!(
            iter.next(),
            Some(&Nugget {
                name: "name2".to_string(),
                kind: NuggetTypeRef::TypeName("uint64_be".to_string()),
            })
        );
        assert_eq!(
            iter.next(),
            Some(&Nugget {
                name: "name3".to_string(),
                kind: NuggetTypeRef::TypeName("f64_le".to_string()),
            })
        );
        assert_eq!(iter.next(), None);
        Ok(())
    }

    #[test]
    fn test_new_type() -> Result<(), CartaError> {
        let tokeniser = Tokeniser::new(
            "struct new_type {
                inner_val1: int8,
                inner_val2: int8,
            }
            val: new_type",
        )?;
        let schema = compile_schema(tokeniser)?;
        let mut iter = schema.nuggets.iter();
        assert_eq!(
            iter.next(),
            Some(&Nugget {
                name: "val".to_string(),
                kind: NuggetTypeRef::TypeName("new_type".to_string()),
            })
        );
        assert_eq!(iter.next(), None);

        let mut types_iter = schema.types.iter();
        assert_eq!(
            types_iter.next(),
            Some(&NuggetStructDefn {
                name: "new_type".to_string(),
                members: vec![
                    Nugget {
                        name: "inner_val1".to_string(),
                        kind: NuggetTypeRef::TypeName("int8".to_string())
                    },
                    Nugget {
                        name: "inner_val2".to_string(),
                        kind: NuggetTypeRef::TypeName("int8".to_string())
                    }
                ]
            })
        );
        assert_eq!(types_iter.next(), None);
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

    #[test]
    fn test_nugget_syntax_errors() -> Result<(), CartaError> {
        let tokeniser = Tokeniser::new("name1, struct1")?;
        let ret = compile_schema(tokeniser);
        assert_eq!(
            ret,
            Err(CartaError::ParseError(":".to_string(), ",".to_string()))
        );

        let tokeniser = Tokeniser::new("name1: struct1,")?;
        let ret = compile_schema(tokeniser);
        assert_eq!(
            ret,
            Err(CartaError::ParseError(
                "<newline>".to_string(),
                ",".to_string()
            ))
        );

        let tokeniser = Tokeniser::new(", struct1")?;
        let ret = compile_schema(tokeniser);
        assert_eq!(
            ret,
            Err(CartaError::ParseError(
                "<name>".to_string(),
                ",".to_string()
            ))
        );
        Ok(())
    }
}
