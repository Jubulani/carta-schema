mod tokeniser;

use std::fs;

use crate::compiler::tokeniser::{Token, TokenType, Tokeniser};

pub struct Schema {
    nuggets: Vec<ILNugget>,
}

impl Schema {
    fn push(&mut self, n: ILNugget) {
        self.nuggets.push(n);
    }
}

impl Schema {
    pub fn iter<'a>(&'a self) -> IterSchema<'a> {
        IterSchema {
            inner: self,
            pos: 0,
        }
    }

    fn get_type(&self, name: &str) -> NuggetType {
        match name {
            "int8" => NuggetType::BuiltinType {
                size: 1,
                name: "int8",
            },
            "int16_be" => NuggetType::BuiltinType {
                size: 2,
                name: "int16_be",
            },
            "int16_le" => NuggetType::BuiltinType {
                size: 2,
                name: "int16_le",
            },
            "int32_be" => NuggetType::BuiltinType {
                size: 4,
                name: "int32_be",
            },
            "int32_le" => NuggetType::BuiltinType {
                size: 4,
                name: "int32_le",
            },
            "int64_be" => NuggetType::BuiltinType {
                size: 8,
                name: "int64_be",
            },
            "int64_le" => NuggetType::BuiltinType {
                size: 8,
                name: "int64_le",
            },
            "uint8" => NuggetType::BuiltinType {
                size: 1,
                name: "uint8",
            },
            "uint16_be" => NuggetType::BuiltinType {
                size: 2,
                name: "uint16_be",
            },
            "uint16_le" => NuggetType::BuiltinType {
                size: 2,
                name: "uint16_le",
            },
            "uint32_be" => NuggetType::BuiltinType {
                size: 4,
                name: "uint32_be",
            },
            "uint32_le" => NuggetType::BuiltinType {
                size: 4,
                name: "uint32_le",
            },
            "uint64_be" => NuggetType::BuiltinType {
                size: 8,
                name: "uint64_be",
            },
            "uint64_le" => NuggetType::BuiltinType {
                size: 8,
                name: "uint64_le",
            },
            "f32_be" => NuggetType::BuiltinType {
                size: 4,
                name: "f32_be",
            },
            "f32_le" => NuggetType::BuiltinType {
                size: 4,
                name: "f32_le",
            },
            "f64_be" => NuggetType::BuiltinType {
                size: 8,
                name: "f64_be",
            },
            "f64_le" => NuggetType::BuiltinType {
                size: 8,
                name: "f64_le",
            },
            _ => panic!("Unrecognised type: {}", name),
        }
    }
}

pub struct IterSchema<'a> {
    inner: &'a Schema,
    pos: usize,
}

impl<'a> Iterator for IterSchema<'a> {
    type Item = &'a ILNugget;

    fn next(&mut self) -> Option<&'a ILNugget> {
        if self.pos < self.inner.nuggets.len() {
            let n = &self.inner.nuggets[self.pos];
            self.pos += 1;
            Some(n)
        } else {
            None
        }
    }
}

#[derive(PartialEq, Debug)]
enum NuggetType {
    UserDefinedType { children: Vec<ILNugget> },
    BuiltinType { size: usize, name: &'static str },
}

#[derive(PartialEq, Debug)]
pub struct ILNugget {
    name: String,
    kind: NuggetType,
}

trait CompilerState {
    fn new_token(self: Box<Self>, t: &Token, schema: &mut Schema) -> Box<dyn CompilerState>;
}

struct EmptyState;

impl CompilerState for EmptyState {
    fn new_token(self: Box<Self>, t: &Token, _: &mut Schema) -> Box<dyn CompilerState> {
        if let Some(s) = new_state(t) {
            return s;
        }
        self
    }
}

struct NewNuggetState {
    state: NewNuggetSubState,
    name: String,
    kind: Option<NuggetType>,
}

#[derive(PartialEq)]
enum NewNuggetSubState {
    Name,
    TypeOf,
    Kind,
}

impl NewNuggetState {
    fn new(t: &Token) -> NewNuggetState {
        NewNuggetState {
            state: NewNuggetSubState::Name,
            name: t.get_value().to_string(),
            kind: None,
        }
    }
}

impl CompilerState for NewNuggetState {
    fn new_token(mut self: Box<Self>, t: &Token, schema: &mut Schema) -> Box<dyn CompilerState> {
        match self.state {
            NewNuggetSubState::Name => {
                // Next state must be TypeOf
                if t.kind != TokenType::TypeOf {
                    panic!("Expected ':', got: {:?}", t);
                }
                self.state = NewNuggetSubState::TypeOf;
            }
            NewNuggetSubState::TypeOf => {
                // Next state must be the type name
                self.kind = Some(schema.get_type(t.get_value()));
                self.state = NewNuggetSubState::Kind;
            }
            NewNuggetSubState::Kind => {
                // Next state must be a newline
                if t.kind != TokenType::NewLine {
                    panic!("Expected '\n', got: {:?}", t);
                }

                if let Some(kind) = self.kind {
                    let nugget = ILNugget {
                        name: self.name,
                        kind,
                    };
                    schema.push(nugget);
                    return Box::new(EmptyState);
                } else {
                    panic!("Kind not available");
                }
            }
        }

        self
    }
}

fn new_state(t: &Token) -> Option<Box<dyn CompilerState>> {
    if t.kind == TokenType::Word {
        return Some(Box::new(NewNuggetState::new(t)));
    } else if t.kind == TokenType::NewLine {
        // Empty newline - nothing to parse
        return None;
    }

    panic!("Unknown token: {:?}", t);
}

pub fn compile_schema_file(filename: &str) -> Schema {
    let s = fs::read_to_string(filename).unwrap();
    compile_schema(&s)
}

fn compile_schema(s: &str) -> Schema {
    let tokeniser = Tokeniser::new(s);

    let mut schema = Schema {
        nuggets: Vec::new(),
    };
    let mut state: Box<dyn CompilerState> = Box::new(EmptyState {});
    for token in tokeniser.iter() {
        state = state.new_token(token, &mut schema);
    }

    // Add a final newline, in case one doesn't exist in the input
    // This will flush any remaining (valid) tokens
    state.new_token(&Token::new(TokenType::NewLine, None), &mut schema);

    schema
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_basic_builtin() {
        let schema = compile_schema("new_name: uint64_le");
        let mut iter = schema.iter();
        assert_eq!(
            iter.next(),
            Some(&ILNugget {
                name: "new_name".to_string(),
                kind: NuggetType::BuiltinType {
                    size: 8,
                    name: "uint64_le"
                },
            })
        );
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_empty_input() {
        let schema = compile_schema("\n  \t\n\n");
        let mut iter = schema.iter();
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_multiple_builtins() {
        let schema = compile_schema(
            "name1: int8
name2: uint64_be

name3: f64_le
",
        );
        let mut iter = schema.iter();
        assert_eq!(
            iter.next(),
            Some(&ILNugget {
                name: "name1".to_string(),
                kind: NuggetType::BuiltinType {
                    size: 1,
                    name: "int8"
                },
            })
        );
        assert_eq!(
            iter.next(),
            Some(&ILNugget {
                name: "name2".to_string(),
                kind: NuggetType::BuiltinType {
                    size: 8,
                    name: "uint64_be"
                },
            })
        );
        assert_eq!(
            iter.next(),
            Some(&ILNugget {
                name: "name3".to_string(),
                kind: NuggetType::BuiltinType {
                    size: 8,
                    name: "f64_le"
                },
            })
        );
        assert_eq!(iter.next(), None);
    }

}
