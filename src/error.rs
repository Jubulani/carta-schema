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

    #[fail(display = "Unexpected symbol: {}, expected {}", _0, _1)]
    UnexpectedSymbol(char, &'static str),

    #[fail(display = "Unclosed block comment at end of file")]
    UnclosedBlockComment(),

    #[fail(display = "Parse error!  Expected '{}', found '{}'", _0, _1)]
    ParseError(String, String),

    #[fail(display = "Missing struct \"root\"")]
    MissingRootElement(),

    #[fail(display = "Bad array length: {}", _0)]
    BadArrayLen(String),

    #[fail(display = "Array length must be builtin integer type: {}", _0)]
    BadArrayLenType(String),

    #[fail(display = "Cannot start number with leading zero")]
    LeadingZero(),

    #[fail(display = "Integer too large: Must be 9 digits or less")]
    IntegerTooLarge(),
}
