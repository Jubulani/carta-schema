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
    ArrayElem(ArrayDefn),
}

#[derive(PartialEq, Debug)]
pub enum ArrayLen {
    Identifier(String),
    Static(u32),
}

#[derive(PartialEq, Debug)]
pub struct ArrayDefn {
    pub kind: String,
    pub length: ArrayLen,
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

    // Default implementation.  Parsing the state hasn't completed, so
    // return an error.  Overwritten in EmptyState to return Ok instead.
    fn eof(self: Box<Self>) -> Result<(), CartaError> {
        Err(CartaError::new_incomplete_input(0))
    }
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

    fn eof(self: Box<Self>) -> Result<(), CartaError> {
        Ok(())
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

    fn append_child(self: &mut Self, kind: ElementTypeRef) {
        let elem = Element {
            name: self.new_child_name.take().unwrap(),
            kind,
        };
        self.complete_children.push(elem);
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
                    return Err(CartaError::new_parse_error(t.line_no, "<name>", t.get_string()));
                }
                self.name = Some(t.get_string());
                self.state = StructSubState::Name;
            }
            StructSubState::Name => {
                // Next token must be OpenBrace
                if t.kind != TokenType::OpenBrace {
                    return Err(CartaError::new_parse_error(t.line_no, "{", t.get_string()));
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
                    self.new_child_name = Some(t.get_string());
                    self.state = StructSubState::ChildName;
                }
                _ => return Err(CartaError::new_parse_error(t.line_no, "}", t.get_string())),
            },
            StructSubState::ChildName => {
                // Next token must be Colon
                if t.kind != TokenType::Colon {
                    return Err(CartaError::new_parse_error(t.line_no, ":", t.get_string()));
                }
                self.state = StructSubState::ChildTypeOf;
            }
            StructSubState::ChildTypeOf => {
                // Next token must be a type definition - a typename or array
                match t.kind {
                    TokenType::Word => {
                        // Typename
                        let kind = ElementTypeRef::TypeName(t.get_string());
                        self.append_child(kind);

                        self.state = StructSubState::ChildKind;
                    }
                    TokenType::OpenBracket => {
                        self.state = StructSubState::ChildKind;
                        let arr = Box::new(ArrayState::new(self));
                        return Ok(arr);
                    }
                    _ => return Err(CartaError::new_parse_error(t.line_no, "<typename>", t.get_string())),
                }
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
                    _ => return Err(CartaError::new_parse_error(t.line_no, ",", t.get_string())),
                }
            }
        }

        Ok(self)
    }
}

struct ArrayState {
    parent: Box<StructState>,
    state: ArraySubState,
    kind: Option<String>,
    length: Option<ArrayLen>
}

impl ArrayState {
    fn new(parent: Box<StructState>) -> ArrayState {
        ArrayState {
            parent,
            state: ArraySubState::Begin,
            kind: None,
            length: None,
        }
    }
}

#[derive(PartialEq)]
enum ArraySubState {
    Begin,
    Kind,
    Semicolon,
    Length,
}

impl CompilerState for ArrayState {
    fn new_token(
        mut self: Box<Self>,
        t: Token,
        _: &mut Schema,
    ) -> Result<Box<dyn CompilerState>, CartaError> {
        // New lines are ignored
        if t.kind == TokenType::NewLine {
            return Ok(self);
        }

        match self.state {
            ArraySubState::Begin => {
                // Firstly, mmust have a typename
                if t.kind != TokenType::Word {
                    return Err(CartaError::new_parse_error(t.line_no, "<typename>", t.get_string()));
                }

                self.kind = Some(t.get_string());
                self.state = ArraySubState::Kind;
            }
            ArraySubState::Kind => {
                // Next is semicolon separating type from length
                if t.kind != TokenType::Semicolon {
                    return Err(CartaError::new_parse_error(t.line_no, ";", t.get_string()));
                }
                self.state = ArraySubState::Semicolon;
            }
            ArraySubState::Semicolon => {
                // Next is length
                match t.kind {
                    TokenType::Word => {
                        self.length = Some(ArrayLen::Identifier(t.get_string()));
                    },
                    TokenType::Integer => {
                        self.length = Some(ArrayLen::Static(t.get_int()));
                    },
                    _ => return Err(CartaError::new_parse_error(t.line_no, "<array_length>", t.get_string())),
                }

                self.state = ArraySubState::Length;
            }
            ArraySubState::Length => {
                // Finally, closing bracket
                if t.kind != TokenType::CloseBracket {
                    return Err(CartaError::new_parse_error(t.line_no, "]", t.get_string()));
                }

                // Aaaand, we're done
                // We know we have both kind and length values available, as we've successfully
                // moved through all states
                let arr_defn = ArrayDefn {
                    kind: self.kind.unwrap(),
                    length: self.length.unwrap(),
                };
                self.parent
                    .append_child(ElementTypeRef::ArrayElem(arr_defn));
                return Ok(self.parent);
            }
        }

        Ok(self)
    }
}

fn new_state(t: Token) -> Result<Option<Box<dyn CompilerState>>, CartaError> {
    if t.kind == TokenType::Word {
        let line_no = t.line_no;  // Copy line_no before consuming t

        // Match against language keywords
        return match t.get_string().as_ref() {
            "struct" => Ok(Some(Box::new(StructState::new()))),
            val => Err(CartaError::new_parse_error(line_no, "<keyword>", val.to_string())),
        };
    } else if t.kind == TokenType::NewLine {
        // Empty newline - nothing to parse
        return Ok(None);
    }

    return Err(CartaError::new_parse_error(t.line_no, "<keyword>", t.get_string()));
}

pub fn compile_schema(tokeniser: Tokeniser) -> Result<Schema, CartaError> {
    let mut schema = Schema {
        structs: Vec::new(),
    };
    let mut state: Box<dyn CompilerState> = Box::new(EmptyState {});
    for token in tokeniser.into_iter() {
        state = state.new_token(token, &mut schema)?;
    }

    // Check that parsing has completed, and we're not waiting for anything else
    state.eof()?;

    Ok(schema)
}

#[cfg(test)]
mod test {
    use super::*;

    fn build_basic_element(name: &str, typename: &str) -> Element {
        Element {
            name: name.to_string(),
            kind: ElementTypeRef::TypeName(typename.to_string()),
        }
    }

    fn build_array_element(name: &str, typename: &str, length: &str) -> Element {
        Element {
            name: name.to_string(),
            kind: ElementTypeRef::ArrayElem(ArrayDefn {
                kind: typename.to_string(),
                length: ArrayLen::Identifier(length.to_string()),
            }),
        }
    }

    fn build_struct(name: &str, elements: Vec<Element>) -> StructDefn {
        StructDefn {
            name: name.to_string(),
            elements: elements,
        }
    }

    #[test]
    fn basic_builtin() -> Result<(), CartaError> {
        let tokeniser = Tokeniser::new("struct s {new_name: uint64_le}")?;
        let schema = compile_schema(tokeniser)?;
        let mut iter = schema.structs.iter();
        assert_eq!(
            iter.next(),
            Some(&build_struct(
                "s",
                vec![build_basic_element("new_name", "uint64_le")]
            ))
        );
        assert_eq!(iter.next(), None);
        Ok(())
    }

    #[test]
    fn empty_input() -> Result<(), CartaError> {
        let tokeniser = Tokeniser::new("")?;
        let schema = compile_schema(tokeniser)?;
        let mut iter = schema.structs.iter();
        assert_eq!(iter.next(), None);
        Ok(())
    }

    #[test]
    fn whitespace_input() -> Result<(), CartaError> {
        let tokeniser = Tokeniser::new("\n  \t\n\n")?;
        let schema = compile_schema(tokeniser)?;
        let mut iter = schema.structs.iter();
        assert_eq!(iter.next(), None);
        Ok(())
    }

    #[test]
    fn multiple_builtins() -> Result<(), CartaError> {
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
                    build_basic_element("name1", "int8"),
                    build_basic_element("name2", "uint64_be"),
                    build_basic_element("name3", "f64_le"),
                ]
            ))
        );
        assert_eq!(iter.next(), None);
        Ok(())
    }

    #[test]
    fn struct_syntax_errors() -> Result<(), CartaError> {
        let tokeniser = Tokeniser::new(
            "struct new_type {
                inner_val1: int8,
                inner_val2, int8,
            }",
        )?;
        let ret = compile_schema(tokeniser);
        assert_eq!(
            ret,
            Err(CartaError::new_parse_error(3, ":", ",".to_string()))
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
            Err(CartaError::new_parse_error(3, "}", ":".to_string()))
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
            Err(CartaError::new_parse_error(1, "<name>", "{".to_string()))
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
            Err(CartaError::new_parse_error(2, "{", "inner_val1".to_string()))
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
            Err(CartaError::new_parse_error(3, "}", ":".to_string()))
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
            Err(CartaError::new_parse_error(2, "<typename>", ",".to_string()))
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
            Err(CartaError::new_parse_error(3, ",", "inner_val2".to_string()))
        );
        Ok(())
    }

    #[test]
    fn array() -> Result<(), CartaError> {
        let tokeniser = Tokeniser::new("struct s {len: int8, arr1: [int8; len]}")?;
        let schema = compile_schema(tokeniser)?;
        let mut iter = schema.structs.iter();
        assert_eq!(
            iter.next(),
            Some(&build_struct(
                "s",
                vec![
                    build_basic_element("len", "int8"),
                    build_array_element("arr1", "int8", "len")
                ]
            ))
        );
        assert_eq!(iter.next(), None);
        Ok(())
    }

    #[test]
    fn array_bad_len() -> Result<(), CartaError> {
        let tokeniser = Tokeniser::new("struct s {len: int8, arr1: [int8; blah]}")?;
        let schema = compile_schema(tokeniser)?;
        let mut iter = schema.structs.iter();
        assert_eq!(
            iter.next(),
            Some(&build_struct(
                "s",
                vec![
                    build_basic_element("len", "int8"),
                    build_array_element("arr1", "int8", "blah")
                ]
            ))
        );
        assert_eq!(iter.next(), None);
        Ok(())
    }

    #[test]
    fn array_static_len() -> Result<(), CartaError> {
        let tokeniser = Tokeniser::new("struct s {arr1: [int8; 4]}")?;
        let schema = compile_schema(tokeniser)?;
        let mut iter = schema.structs.iter();
        assert_eq!(
            iter.next(),
            Some(&build_struct(
                "s",
                vec![
                    Element {
                        name: "arr1".to_string(),
                        kind: ElementTypeRef::ArrayElem(ArrayDefn {
                            kind: "int8".to_string(),
                            length: ArrayLen::Static(4),
                        }),
                    }
                ]
            ))
        );
        assert_eq!(iter.next(), None);
        Ok(())
    }

    #[test]
    fn incomplete_input() {
        let tokeniser = Tokeniser::new("struct s {field_1").unwrap();
        let result = compile_schema(tokeniser);
        assert_eq!(result, Err(CartaError::new_incomplete_input(0)));
    }

}
