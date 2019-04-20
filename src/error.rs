use failure_derive::Fail;

#[derive(Fail, Debug, PartialEq)]
pub enum CartaError {
    #[fail(display = "Unrecognized type: {}", _0)]
    UnknownType(String),

    #[fail(display = "Duplicate definition for type: {}", _0)]
    DuplicateType(String),

    #[fail(display = "Recursive types: {:?}", _0)]
    RecursiveTypes(Vec<String>),

    #[fail(display = "Unrecognized symbol: {}", _0)]
    UnknownSymbol(char),

    #[fail(display = "Parse error!  Expected '{}', found '{}'", _0, _1)]
    ParseError(String, String),

    #[fail(display = "Missing struct \"root\"")]
    MissingRootElement(),
}
