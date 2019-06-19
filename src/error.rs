use failure_derive::Fail;

#[derive(Fail, Debug, PartialEq)]
#[fail(display = "Line {}: {}", _0, _1)]
pub struct CartaError {
    pub line_no: usize,
    pub code: CartaErrorCode
}

#[derive(Fail, Debug, PartialEq)]
pub enum CartaErrorCode {
    #[fail(display = "Unrecognized type: {}", _0)]
    UnknownType(String),

    #[fail(display = "Duplicate definition for type: {}", _0)]
    DuplicateType(String),

    #[fail(display = "Recursive types: {:?}", _0)]
    RecursiveTypes(Vec<String>),

    #[fail(display = "Unrecognized symbol: {}", _0)]
    UnknownSymbol(char),

    #[fail(display = "Unexpected symbol.  Expected {}, found: {}", _0, _1)]
    UnexpectedSymbol(&'static str, char),

    #[fail(display = "Unclosed block comment at end of file")]
    UnclosedBlockComment(),

    #[fail(display = "Parse error!  Expected '{}', found '{}'", _0, _1)]
    ParseError(&'static str, String),

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

// Make errors slightly easier to construct
impl CartaError {
    pub fn new_unknown_type(line_no: usize, kind: String) -> CartaError {
        CartaError {
            line_no,
            code: CartaErrorCode::UnknownType(kind),
        }
    }

    pub fn new_duplicate_type(line_no: usize, kind: String) -> CartaError {
        CartaError {
            line_no,
            code: CartaErrorCode::DuplicateType(kind),
        }
    }

    pub fn new_recursive_types(line_no: usize, kinds: Vec<String>) -> CartaError {
        CartaError {
            line_no,
            code: CartaErrorCode::RecursiveTypes(kinds),
        }
    }

    pub fn new_unknown_symbol(line_no: usize, sym: char) -> CartaError {
        CartaError {
            line_no,
            code: CartaErrorCode::UnknownSymbol(sym),
        }
    }

    pub fn new_unexpected_symbol(line_no: usize, expected: &'static str, got: char) -> CartaError {
        CartaError {
            line_no,
            code: CartaErrorCode::UnexpectedSymbol(expected, got),
        }
    }

    pub fn new_unclosed_block_comment(line_no: usize) -> CartaError {
        CartaError {
            line_no,
            code: CartaErrorCode::UnclosedBlockComment(),
        }
    }

    pub fn new_parse_error(line_no: usize, expected: &'static str, got: String) -> CartaError {
        CartaError {
            line_no,
            code: CartaErrorCode::ParseError(expected, got),
        }
    }

    pub fn new_missing_root_element(line_no: usize) -> CartaError {
        CartaError {
            line_no,
            code: CartaErrorCode::MissingRootElement(),
        }
    }

    pub fn new_bad_array_len(line_no: usize, len_desc: &str) -> CartaError {
        CartaError {
            line_no,
            code: CartaErrorCode::BadArrayLen(len_desc.to_string()),
        }
    }

    pub fn new_bad_array_len_type(line_no: usize, kind: &str) -> CartaError {
        CartaError {
            line_no,
            code: CartaErrorCode::BadArrayLenType(kind.to_string()),
        }
    }

    pub fn new_leading_zero(line_no: usize) -> CartaError {
        CartaError {
            line_no,
            code: CartaErrorCode::LeadingZero(),
        }
    }

    pub fn new_integer_too_large(line_no: usize) -> CartaError {
        CartaError {
            line_no,
            code: CartaErrorCode::IntegerTooLarge(),
        }
    }
}