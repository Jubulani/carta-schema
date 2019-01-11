use crate::compiler::tokeniser::{Token, TokenType, Tokeniser};

pub struct Schema {
    nuggets: Vec<ILNugget>,
    types: Vec<NuggetStructDefn>,
}

impl Schema {
    pub fn iter<'a>(&'a self) -> Iter<'a> {
        Iter {
            inner: self,
            pos: 0,
        }
    }

    fn add_nugget(&mut self, n: ILNugget) {
        self.nuggets.push(n);
    }

    fn add_struct(&mut self, s: NuggetStructDefn) {
        self.types.push(s);
    }

    /*fn get_type(&self, name: &str) -> NuggetType {
        let (size, kind) = types::get_type(name);
        NuggetType::SimpleType { size, kind }
    }*/
}

pub struct Iter<'a> {
    inner: &'a Schema,
    pos: usize,
}

impl<'a> Iterator for Iter<'a> {
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
pub struct ILNugget {
    name: String,
    kind: NuggetTypeRef,
}

#[derive(PartialEq, Debug)]
enum NuggetTypeRef {
    TypeName(String),
}

#[derive(PartialEq, Debug)]
struct NuggetStructDefn {
    name: String,
    members: Vec<ILNugget>,
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
    kind: Option<NuggetTypeRef>,
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
            name: t.value().to_string(),
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
                self.kind = Some(NuggetTypeRef::TypeName(t.value().to_string()));
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
                    schema.add_nugget(nugget);
                    return Box::new(EmptyState);
                } else {
                    panic!("Kind not available");
                }
            }
        }

        self
    }
}

struct StructState {
    state: StructSubState,
    name: Option<String>,
    complete_children: Vec<ILNugget>,
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
    fn new_token(mut self: Box<Self>, t: &Token, schema: &mut Schema) -> Box<dyn CompilerState> {
        // New lines are ignored in struct definitions
        if t.kind == TokenType::NewLine {
            return self;
        }

        match self.state {
            StructSubState::Begin => {
                if t.kind != TokenType::Word {
                    panic!("Expected name, got: {:?}", t);
                }
                self.name = Some(t.value().to_string());
                self.state = StructSubState::Name;
            }
            StructSubState::Name => {
                // Next state must be OpenBrace
                if t.kind != TokenType::OpenBrace {
                    panic!("Expected '{{', got: {:?}", t);
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
                    return Box::new(EmptyState {});
                }
                TokenType::Word => {
                    self.new_child_name = Some(t.value().to_string());
                    self.state = StructSubState::ChildName;
                }
                _ => panic!("Expected '}}' or name, got: {:?}", t),
            },
            StructSubState::ChildName => {
                // Next state must be TypeOf
                if t.kind != TokenType::TypeOf {
                    panic!("Expected ':', got: {:?}", t);
                }
                self.state = StructSubState::ChildTypeOf;
            }
            StructSubState::ChildTypeOf => {
                // Next state must be a typename
                if t.kind != TokenType::Word {
                    panic!("Expected type name, got: {:?}", t);
                }
                let kind = t.value().to_string();
                self.complete_children.push(ILNugget {
                    name: self.new_child_name.take().unwrap(),
                    kind: NuggetTypeRef::TypeName(kind),
                });
                self.state = StructSubState::ChildKind;
            }
            StructSubState::ChildKind => {
                // Next state must be Comma
                if t.kind != TokenType::Comma {
                    panic!("Expected ',', got: {:?}", t);
                }
                self.state = StructSubState::OpenBrace;
            }
        }

        self
    }
}

fn new_state(t: &Token) -> Option<Box<dyn CompilerState>> {
    if t.kind == TokenType::Word {
        // Match against language keywords
        return match t.value() {
            "struct" => Some(Box::new(StructState::new())),

            // If not a keyword, must be a new nugget name
            _ => Some(Box::new(NewNuggetState::new(t))),
        };
    } else if t.kind == TokenType::NewLine {
        // Empty newline - nothing to parse
        return None;
    }

    panic!("Unknown token: {:?}", t);
}

pub fn compile_schema(tokeniser: &Tokeniser) -> Schema {
    let mut schema = Schema {
        nuggets: Vec::new(),
        types: Vec::new(),
    };
    let mut state: Box<dyn CompilerState> = Box::new(EmptyState {});
    for token in tokeniser.iter() {
        state = state.new_token(token, &mut schema);
    }

    // Add a final newline, in case one doesn't exist in the input
    // This will flush any remaining (valid) nuggets
    state.new_token(&Token::new(TokenType::NewLine, None), &mut schema);

    // TODO: Handle incomplete nuggets still remaining.

    schema
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_basic_builtin() {
        let tokeniser = Tokeniser::new("new_name: uint64_le");
        let schema = compile_schema(&tokeniser);
        let mut iter = schema.iter();
        assert_eq!(
            iter.next(),
            Some(&ILNugget {
                name: "new_name".to_string(),
                kind: NuggetTypeRef::TypeName("uint64_le".to_string()),
            })
        );
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_empty_input() {
        let tokeniser = Tokeniser::new("");
        let schema = compile_schema(&tokeniser);
        let mut iter = schema.iter();
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_whitespace_input() {
        let tokeniser = Tokeniser::new("\n  \t\n\n");
        let schema = compile_schema(&tokeniser);
        let mut iter = schema.iter();
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_multiple_builtins() {
        let tokeniser = Tokeniser::new("name1: int8
            name2: uint64_be
            name3: f64_le
        ");
        let schema = compile_schema(&tokeniser);
        let mut iter = schema.iter();
        assert_eq!(
            iter.next(),
            Some(&ILNugget {
                name: "name1".to_string(),
                kind: NuggetTypeRef::TypeName("int8".to_string()),
            })
        );
        assert_eq!(
            iter.next(),
            Some(&ILNugget {
                name: "name2".to_string(),
                kind: NuggetTypeRef::TypeName("uint64_be".to_string()),
            })
        );
        assert_eq!(
            iter.next(),
            Some(&ILNugget {
                name: "name3".to_string(),
                kind: NuggetTypeRef::TypeName("f64_le".to_string()),
            })
        );
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_new_type() {
        let tokeniser = Tokeniser::new("struct new_type {
                inner_val1: int8,
                inner_val2: int8,
            }
            val: new_type");
        let schema = compile_schema(&tokeniser);
        let mut iter = schema.iter();
        assert_eq!(
            iter.next(),
            Some(&ILNugget {
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
                    ILNugget {
                        name: "inner_val1".to_string(),
                        kind: NuggetTypeRef::TypeName("int8".to_string())
                    },
                    ILNugget {
                        name: "inner_val2".to_string(),
                        kind: NuggetTypeRef::TypeName("int8".to_string())
                    }
                ]
            })
        );
        assert_eq!(types_iter.next(), None);
    }
}
