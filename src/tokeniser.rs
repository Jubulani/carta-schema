/*!
 * Tokeniser
 *
 * This is the first stage of the compilation pipline.  It returns a Tokeniser struct which can
 * be used to iterate over all tokens in the input string.
 *
 * We generate tokens by iterating over the input character-by-character in one pass, using a
 * state machine to save current token state.
 *
 * Tests at the end of the file show examples of tokens we get from various input strings.
 */

use crate::error::CartaError;

#[derive(PartialEq, Debug, Clone)]
pub enum TokenType {
    Word,         // Start with _ or letter, can contain any number of _, letter or digit after that
    Colon,        // :
    NewLine,      // \n
    OpenBrace,    // {
    CloseBrace,   // }
    Comma,        // ,
    OpenBracket,  // [
    CloseBracket, // ]
    Semicolon,    // ;
    Integer,      // Starts with 1-9, continues with any digit.  Max 9 digits, to guarantee that it will
                  // always fit into a u32
}

#[derive(PartialEq, Debug, Clone)]
pub struct Token {
    pub kind: TokenType,
    line_no: usize,
    value: TokenValue,
}

#[derive(PartialEq, Debug, Clone)]
enum TokenValue {
    StringVal(String),
    IntVal(u32),
}

trait IntoTokenValue {
    fn into_tokenvalue(self) -> TokenValue;
}

impl IntoTokenValue for String {
    fn into_tokenvalue(self) -> TokenValue {
        TokenValue::StringVal(self)
    }
}

impl IntoTokenValue for u32 {
    fn into_tokenvalue(self) -> TokenValue {
        TokenValue::IntVal(self)
    }
}

impl Token {
    fn new<V: IntoTokenValue>(kind: TokenType, value: V, line_no: usize) -> Token {
        Token { kind, value: value.into_tokenvalue(), line_no }
    }

    pub fn get_string(self: Self) -> String {
        match self.value {
            TokenValue::StringVal(sval) => sval,
            TokenValue::IntVal(ival) => ival.to_string(),
        }
    }

    pub fn get_int(self: Self) -> u32 {
        match self.value {
            TokenValue::StringVal(_) => panic!("Expected int, got String in token value"),
            TokenValue::IntVal(i) => i
        }
    }
}

#[derive(PartialEq, Debug)]
pub struct Tokeniser {
    tokens: Vec<Token>,
}

impl Tokeniser {
    /// Iterate over the input by character, and generate a list of output tokens
    pub fn new(data: &str) -> Result<Tokeniser, CartaError> {
        let mut tokens: Vec<Token> = Vec::new();
        let mut state: Box<dyn TokeniserState> = Box::new(EmptyState {});
        let mut line_no = 1;

        for c in data.chars() {
            state = state.new_char(c, &mut tokens, line_no)?;
            // Newlines count as being on the line they end, not the new line they start
            if c == '\n' {
                line_no += 1;
            }
        }

        // Once we're done with the input, we may still be in the process of building a token.  If we are,
        // and it's valid, add it to the list.
        if let Some(t) = state.eof()? {
            tokens.push(t);
        }
        Ok(Tokeniser { tokens })
    }

    pub fn into_iter(self) -> std::vec::IntoIter<Token> {
        self.tokens.into_iter()
    }
}

/// State machine to handle emitting tokens based on input
trait TokeniserState {
    /// Process a new input character `c`.  Maybe emit token(s) by appending to `tokens`.  Return the next state.
    fn new_char(
        self: Box<Self>,
        c: char,
        tokens: &mut Vec<Token>,
        line_no: usize,
    ) -> Result<Box<dyn TokeniserState>, CartaError>;

    /// Pick up the final token if there is one waiting to be emmitted on end-of-input.
    fn eof(self: Box<Self>) -> Result<Option<Token>, CartaError>;
}

/// Default state representing start of input, or when previous input has all been completely
/// consumed.
struct EmptyState;

impl TokeniserState for EmptyState {
    fn new_char(
        self: Box<Self>,
        c: char,
        tokens: &mut Vec<Token>,
        line_no: usize,
    ) -> Result<Box<dyn TokeniserState>, CartaError> {
        if let Some(s) = new_state(c, tokens, line_no)? {
            return Ok(s);
        }
        Ok(self)
    }

    fn eof(self: Box<Self>) -> Result<Option<Token>, CartaError> {
        Ok(None)
    }
}

/// State representing processing of a `TokenType::Word`
struct WordState {
    value: String,
    line_no: usize,
}

impl WordState {
    fn new(c: char, line_no: usize) -> WordState {
        WordState {
            value: c.to_string(),
            line_no
        }
    }

    fn get_token(self: Box<Self>) -> Token {
        Token::new(TokenType::Word, self.value, self.line_no)
    }
}

impl TokeniserState for WordState {
    fn new_char(
        mut self: Box<Self>,
        c: char,
        tokens: &mut Vec<Token>,
        line_no: usize,
    ) -> Result<Box<dyn TokeniserState>, CartaError> {
        // Accept the token, and add it to the saved token value
        if c.is_alphabetic() || c.is_numeric() || c == '_' {
            self.value.push(c);
            Ok(self)
        } else {
            // Otherwise, the next character is not a valid word character.  Emit the Word found so far,
            // and continue processing the input character as a potential new unknown token.
            tokens.push(self.get_token());

            if let Some(s) = new_state(c, tokens, line_no)? {
                Ok(s)
            } else {
                Ok(Box::new(EmptyState))
            }
        }
    }

    fn eof(self: Box<Self>) -> Result<Option<Token>, CartaError> {
        Ok(Some(self.get_token()))
    }
}

struct IntegerState {
    value: u32,
    num_digits: usize,
    line_no: usize,
}

impl IntegerState {
    fn new(c: char, line_no: usize) -> Result<IntegerState, CartaError> {
        let val = c.to_digit(10).unwrap();
        // can't start with a 0
        if val == 0 {
            return Err(CartaError::new_leading_zero(0));
        }
        Ok(IntegerState {
            value: val,
            num_digits: 1,
            line_no
        })
    }

    fn get_token(self: Box<Self>) -> Token {
        Token::new(TokenType::Integer, self.value, self.line_no)
    }
}

impl TokeniserState for IntegerState {
    fn new_char(
        mut self: Box<Self>,
        c: char,
        tokens: &mut Vec<Token>,
        line_no: usize,
    ) -> Result<Box<dyn TokeniserState>, CartaError> {
        if let Some(new_val) = c.to_digit(10) {
            // Check we will still be in bounds
            if self.num_digits > 8 {
                Err(CartaError::new_integer_too_large(0))
            } else {
                self.value *= 10;
                self.value += new_val;
                self.num_digits += 1;
                Ok(self)
            }
        } else {
            // We've finished the integer
            tokens.push(self.get_token());

            if let Some(s) = new_state(c, tokens, line_no)? {
                Ok(s)
            } else {
                Ok(Box::new(EmptyState))
            }
        }
    }

    fn eof(self: Box<Self>) -> Result<Option<Token>, CartaError> {
        Ok(Some(self.get_token()))
    }
}

struct CommentState; // Don't yet know if it's a block comment or a line comment

impl TokeniserState for CommentState {
    fn new_char(
        self: Box<Self>,
        c: char,
        _: &mut Vec<Token>,
        _: usize,
    ) -> Result<Box<dyn TokeniserState>, CartaError> {
        // Decide between a block comment and a line comment
        return match c {
            '/' => Ok(Box::new(LineCommentState)),
            '*' => Ok(Box::new(BlockCommentState)),
            _ => Err(CartaError::new_unexpected_symbol(0, "* or /", c)),
        };
    }

    fn eof(self: Box<Self>) -> Result<Option<Token>, CartaError> {
        Ok(None)
    }
}

struct LineCommentState;

impl TokeniserState for LineCommentState {
    fn new_char(
        self: Box<Self>,
        c: char,
        _: &mut Vec<Token>,
        _: usize,
    ) -> Result<Box<dyn TokeniserState>, CartaError> {
        if c == '\n' {
            // Newline.  End of comment.
            return Ok(Box::new(EmptyState));
        } else {
            return Ok(self);
        }
    }

    // Allow eof in line comments
    fn eof(self: Box<Self>) -> Result<Option<Token>, CartaError> {
        Ok(None)
    }
}

struct BlockCommentState;

impl TokeniserState for BlockCommentState {
    fn new_char(
        self: Box<Self>,
        c: char,
        _: &mut Vec<Token>,
        _: usize,
    ) -> Result<Box<dyn TokeniserState>, CartaError> {
        if c == '*' {
            // Maybe end of comment
            return Ok(Box::new(EndBlockCommentState));
        } else {
            return Ok(self);
        }
    }

    fn eof(self: Box<Self>) -> Result<Option<Token>, CartaError> {
        Err(CartaError::new_unclosed_block_comment(0))
    }
}

struct EndBlockCommentState;

impl TokeniserState for EndBlockCommentState {
    fn new_char(
        self: Box<Self>,
        c: char,
        _: &mut Vec<Token>,
        _: usize,
    ) -> Result<Box<dyn TokeniserState>, CartaError> {
        if c == '/' {
            // End of comment
            return Ok(Box::new(EmptyState));
        } else {
            // Wasn't end of comment after all.  Comment continues
            return Ok(Box::new(BlockCommentState));
        }
    }

    fn eof(self: Box<Self>) -> Result<Option<Token>, CartaError> {
        Err(CartaError::new_unclosed_block_comment(0))
    }
}

/// Process a new input character with no current state.  Matched single-character tokens are
/// emitted immediately, matched multi character tokens return the appropriate state.
fn new_state(
    c: char,
    tokens: &mut Vec<Token>,
    line_no: usize,
) -> Result<Option<Box<dyn TokeniserState>>, CartaError> {
    if c == '\n' {
        tokens.push(Token::new(TokenType::NewLine, c.to_string(), line_no));
        return Ok(None);
    }

    // Whitespace (except newlines) is only used for delimiting tokens.  Ignore it.
    if c.is_whitespace() {
        return Ok(None);
    }

    // Word tokens start with a letter or underscore.
    if c.is_alphabetic() || c == '_' {
        return Ok(Some(Box::new(WordState::new(c, line_no))));
    }

    if c.is_digit(10) {
        return Ok(Some(Box::new(IntegerState::new(c, line_no)?)));
    }

    match c {
        ':' => tokens.push(Token::new(TokenType::Colon, c.to_string(), line_no)),
        '{' => tokens.push(Token::new(TokenType::OpenBrace, c.to_string(), line_no)),
        '}' => tokens.push(Token::new(TokenType::CloseBrace, c.to_string(), line_no)),
        ',' => tokens.push(Token::new(TokenType::Comma, c.to_string(), line_no)),
        '[' => tokens.push(Token::new(TokenType::OpenBracket, c.to_string(), line_no)),
        ']' => tokens.push(Token::new(TokenType::CloseBracket, c.to_string(), line_no)),
        ';' => tokens.push(Token::new(TokenType::Semicolon, c.to_string(), line_no)),
        '/' => return Ok(Some(Box::new(CommentState))), // Start a comment
        _ => return Err(CartaError::new_unknown_symbol(0, c)),
    }

    return Ok(None);
}

#[cfg(test)]
mod test {
    use super::*;

    impl IntoTokenValue for &str {
        fn into_tokenvalue(self) -> TokenValue {
            TokenValue::StringVal(self.to_string())
        }
    }

    fn token<V: IntoTokenValue>(kind: TokenType, val: V, line_no: usize) -> Option<Token> {
        Some(Token {
            kind,
            value: val.into_tokenvalue(),
            line_no,
        })
    }

    #[test]
    fn tokenise_word() -> Result<(), CartaError> {
        let tok = Tokeniser::new("abc")?;
        let mut iter = tok.into_iter();
        assert_eq!(
            iter.next(),
            token(TokenType::Word, "abc", 1)
        );
        assert_eq!(iter.next(), None);
        Ok(())
    }

    #[test]
    fn space_at_start_plus_number() -> Result<(), CartaError> {
        let tok = Tokeniser::new(" abc23")?;
        let mut iter = tok.into_iter();
        assert_eq!(
            iter.next(),
            token(TokenType::Word, "abc23", 1)
        );
        assert_eq!(iter.next(), None);
        Ok(())
    }

    #[test]
    fn newline_at_start_plus_underscore() -> Result<(), CartaError> {
        let tok = Tokeniser::new("\n_abc_abc")?;
        let mut iter = tok.into_iter();
        assert_eq!(
            iter.next(),
            token(TokenType::NewLine, "\n", 1)
        );
        assert_eq!(
            iter.next(),
            token(TokenType::Word, "_abc_abc", 2)
        );
        assert_eq!(iter.next(), None);
        Ok(())
    }

    #[test]
    fn whitespace_at_start_tab() -> Result<(), CartaError> {
        let tok = Tokeniser::new("\tabc")?;
        let mut iter = tok.into_iter();
        assert_eq!(
            iter.next(),
            token(TokenType::Word, "abc", 1)
        );
        assert_eq!(iter.next(), None);
        Ok(())
    }

    #[test]
    fn multiple_words() -> Result<(), CartaError> {
        let tok = Tokeniser::new("abc def\nghi\tjkl ")?;
        let mut iter = tok.into_iter();
        assert_eq!(
            iter.next(),
            token(TokenType::Word, "abc", 1)
        );
        assert_eq!(
            iter.next(),
            token(TokenType::Word, "def", 1)
        );
        assert_eq!(
            iter.next(),
            token(TokenType::NewLine, "\n", 1)
        );
        assert_eq!(
            iter.next(),
            token(TokenType::Word, "ghi", 2)
        );
        assert_eq!(
            iter.next(),
            token(TokenType::Word, "jkl", 2)
        );
        assert_eq!(iter.next(), None);
        Ok(())
    }

    #[test]
    fn basic_typeof() -> Result<(), CartaError> {
        let tok = Tokeniser::new("abc: uint64_le")?;
        let mut iter = tok.into_iter();
        assert_eq!(
            iter.next(),
            token(TokenType::Word, "abc", 1)
        );
        assert_eq!(
            iter.next(),
            token(TokenType::Colon, ":", 1)
        );
        assert_eq!(
            iter.next(),
            token(TokenType::Word, "uint64_le", 1)
        );
        assert_eq!(iter.next(), None);
        Ok(())
    }

    #[test]
    fn basic_struct() -> Result<(), CartaError> {
        let tok = Tokeniser::new(
            "
        struct new_type {
            val1: type1,
            val2: type2
        }",
        )?;
        let mut iter = tok.into_iter();
        assert_eq!(
            iter.next(),
            token(TokenType::NewLine, "\n", 1)
        );
        assert_eq!(
            iter.next(),
            token(TokenType::Word, "struct", 2)
        );
        assert_eq!(
            iter.next(),
            token(TokenType::Word, "new_type", 2)
        );
        assert_eq!(
            iter.next(),
            token(TokenType::OpenBrace, "{", 2)
        );
        assert_eq!(
            iter.next(),
            token(TokenType::NewLine, "\n", 2)
        );
        assert_eq!(
            iter.next(),
            token(TokenType::Word, "val1", 3)
        );
        assert_eq!(
            iter.next(),
            token(TokenType::Colon, ":", 3)
        );
        assert_eq!(
            iter.next(),
            token(TokenType::Word, "type1", 3)
        );
        assert_eq!(
            iter.next(),
            token(TokenType::Comma, ",", 3)
        );
        assert_eq!(
            iter.next(),
            token(TokenType::NewLine, "\n", 3)
        );
        assert_eq!(
            iter.next(),
            token(TokenType::Word, "val2", 4)
        );
        assert_eq!(
            iter.next(),
            token(TokenType::Colon, ":", 4)
        );
        assert_eq!(
            iter.next(),
            token(TokenType::Word, "type2", 4)
        );
        assert_eq!(
            iter.next(),
            token(TokenType::NewLine, "\n", 4)
        );
        assert_eq!(
            iter.next(),
            token(TokenType::CloseBrace, "}", 5)
        );
        Ok(())
    }

    #[test]
    fn unknown_token() {
        let tok = Tokeniser::new("\tabc😃");
        assert_eq!(tok, Err(CartaError::new_unknown_symbol(0, '😃')));
    }

    #[test]
    fn line_comment() -> Result<(), CartaError> {
        let tok = Tokeniser::new("//\nabc//xyz\n")?;
        let mut iter = tok.into_iter();
        assert_eq!(
            iter.next(),
            token(TokenType::Word, "abc", 2)
        );
        assert_eq!(iter.next(), None);
        Ok(())
    }

    #[test]
    fn block_comment() -> Result<(), CartaError> {
        let tok = Tokeniser::new("/*abc*/abc/**/")?;
        let mut iter = tok.into_iter();
        assert_eq!(
            iter.next(),
            token(TokenType::Word, "abc", 1)
        );
        assert_eq!(iter.next(), None);
        Ok(())
    }

    #[test]
    fn incomplete_block_comment() {
        let tok = Tokeniser::new("/*");
        assert_eq!(tok, Err(CartaError::new_unclosed_block_comment(0)));
    }

    #[test]
    fn integer() -> Result<(), CartaError> {
        let tok = Tokeniser::new("123456789")?;
        let mut iter = tok.into_iter();
        assert_eq!(
            iter.next(),
            token(TokenType::Integer, 123456789, 1)
        );
        assert_eq!(iter.next(), None);
        Ok(())
    }

    #[test]
    fn large_integer() {
        let tok = Tokeniser::new("1000000000");
        assert_eq!(tok, Err(CartaError::new_integer_too_large(0)));
    }

    #[test]
    fn leading_zero() {
        let tok = Tokeniser::new("01");
        assert_eq!(tok, Err(CartaError::new_leading_zero(0)));
    }
}
