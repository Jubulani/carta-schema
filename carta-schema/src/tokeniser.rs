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

#[derive(PartialEq, Debug, Clone)]
pub enum TokenType {
    Word,       // Start with _ or letter, can contain any number of _, letter or digit after that
    TypeOf,     // :
    NewLine,    // \n
    OpenBrace,  // {
    CloseBrace, // }
    Comma,      // ,
}

#[derive(PartialEq, Debug, Clone)]
pub struct Token {
    pub kind: TokenType,
    value: Option<String>,
}

impl Token {
    pub fn new(kind: TokenType, value: Option<String>) -> Token {
        Token { kind, value }
    }

    /**
     * Get a token value.  Certain tokens (eg Word) must have a value, other tokens cannot.
     */
    pub fn value(&self) -> &str {
        if let Some(val) = &self.value {
            &val
        } else {
            panic!("Expected value from token: {:?}", self);
        }
    }
}

pub struct Tokeniser {
    tokens: Vec<Token>,
}

impl Tokeniser {
    /// Iterate over the input by character, and generate a list of output tokens
    pub fn new(data: &str) -> Tokeniser {
        let mut tokens: Vec<Token> = Vec::new();
        let mut state: Box<dyn TokeniserState> = Box::new(EmptyState {});
        for c in data.chars() {
            state = state.new_char(c, &mut tokens);
        }

        // Once we're done with the input, we may still be in the process of building a token.  If we are,
        // add it to the list.
        if let Some(t) = state.get_token() {
            tokens.push(t);
        }
        Tokeniser { tokens }
    }

    pub fn iter(&self) -> Iter {
        Iter {
            inner: self,
            pos: 0,
        }
    }
}

pub struct Iter<'a> {
    inner: &'a Tokeniser,
    pos: usize,
}

impl<'a> Iterator for Iter<'a> {
    type Item = &'a Token;

    fn next(&mut self) -> Option<&'a Token> {
        if self.pos < self.inner.tokens.len() {
            let t = &self.inner.tokens[self.pos];
            self.pos += 1;
            Some(t)
        } else {
            None
        }
    }
}

/// State machine to handle emitting tokens based on input
trait TokeniserState {
    /// Process a new input character `c`.  Maybe emit token(s) by appending to `tokens`.  Return the next state.
    fn new_char(self: Box<Self>, c: char, tokens: &mut Vec<Token>) -> Box<dyn TokeniserState>;

    /// Get an incomplete token if one exists.  Used to get the last token when the input doesn't
    /// end with whitespace.
    fn get_token(self: Box<Self>) -> Option<Token>;
}

/// Default state representing start of input, or when previous input has all been completely
/// consumed.
struct EmptyState;

impl TokeniserState for EmptyState {
    fn new_char(self: Box<Self>, c: char, tokens: &mut Vec<Token>) -> Box<dyn TokeniserState> {
        if let Some(s) = new_state(c, tokens) {
            return s;
        }
        self
    }

    fn get_token(self: Box<Self>) -> Option<Token> {
        None
    }
}

/// State representing processing of a `TokenType::Word`
struct WordState {
    value: String,
}

impl WordState {
    fn new(c: char) -> WordState {
        WordState {
            value: c.to_string(),
        }
    }
}

impl TokeniserState for WordState {
    fn new_char(mut self: Box<Self>, c: char, tokens: &mut Vec<Token>) -> Box<dyn TokeniserState> {
        // Accept the token, and add it to the saved token value
        if c.is_alphabetic() || c.is_numeric() || c == '_' {
            self.value.push(c);
            self
        } else {
            // Otherwise, the next character is not a valid word character.  Emit the Word found so far,
            // and continue processing the input character as a potential new unknown token.
            tokens.push(self.get_token().unwrap());

            if let Some(s) = new_state(c, tokens) {
                s
            } else {
                Box::new(EmptyState {})
            }
        }
    }

    fn get_token(self: Box<Self>) -> Option<Token> {
        Some(Token::new(TokenType::Word, Some(self.value)))
    }
}

/// Process a new input character with no current state.  Matched single-character tokens are
/// emitted immediately, matched multi character tokens return the appropriate state.
fn new_state(c: char, tokens: &mut Vec<Token>) -> Option<Box<dyn TokeniserState>> {
    if c == '\n' {
        tokens.push(Token::new(TokenType::NewLine, None));
        return None;
    }

    // Whitespace (except newlines) is only used for delimiting tokens.  Ignore it.
    if c.is_whitespace() {
        return None;
    }

    // Word tokens start with a letter or underscore.
    if c.is_alphabetic() || c == '_' {
        return Some(Box::new(WordState::new(c)));
    }

    // No state needed
    if c == ':' {
        tokens.push(Token::new(TokenType::TypeOf, None));
        return None;
    }
    if c == '{' {
        tokens.push(Token::new(TokenType::OpenBrace, None));
        return None;
    }
    if c == '}' {
        tokens.push(Token::new(TokenType::CloseBrace, None));
        return None;
    }
    if c == ',' {
        tokens.push(Token::new(TokenType::Comma, None));
        return None;
    }

    // TODO: Better error handling
    panic!("Unknown symbol: {}", c);
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_tokenise_word() {
        let tok = Tokeniser::new("abc");
        let mut iter = tok.iter();
        assert_eq!(
            iter.next(),
            Some(&Token {
                kind: TokenType::Word,
                value: Some("abc".to_string())
            })
        );
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_space_at_start_plus_number() {
        let tok = Tokeniser::new(" abc23");
        let mut iter = tok.iter();
        assert_eq!(
            iter.next(),
            Some(&Token {
                kind: TokenType::Word,
                value: Some("abc23".to_string())
            })
        );
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_newline_at_start_plus_underscore() {
        let tok = Tokeniser::new("\n_abc_abc");
        let mut iter = tok.iter();
        assert_eq!(
            iter.next(),
            Some(&Token {
                kind: TokenType::NewLine,
                value: None
            })
        );
        assert_eq!(
            iter.next(),
            Some(&Token {
                kind: TokenType::Word,
                value: Some("_abc_abc".to_string())
            })
        );
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_whitespace_at_start_tab() {
        let tok = Tokeniser::new("\tabc");
        let mut iter = tok.iter();
        assert_eq!(
            iter.next(),
            Some(&Token {
                kind: TokenType::Word,
                value: Some("abc".to_string())
            })
        );
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_multiple_words() {
        let tok = Tokeniser::new("abc def\nghi\tjkl ");
        let mut iter = tok.iter();
        assert_eq!(
            iter.next(),
            Some(&Token {
                kind: TokenType::Word,
                value: Some("abc".to_string())
            })
        );
        assert_eq!(
            iter.next(),
            Some(&Token {
                kind: TokenType::Word,
                value: Some("def".to_string())
            })
        );
        assert_eq!(
            iter.next(),
            Some(&Token {
                kind: TokenType::NewLine,
                value: None
            })
        );
        assert_eq!(
            iter.next(),
            Some(&Token {
                kind: TokenType::Word,
                value: Some("ghi".to_string())
            })
        );
        assert_eq!(
            iter.next(),
            Some(&Token {
                kind: TokenType::Word,
                value: Some("jkl".to_string())
            })
        );
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_basic_typeof() {
        let tok = Tokeniser::new("abc: uint64_le");
        let mut iter = tok.iter();
        assert_eq!(
            iter.next(),
            Some(&Token {
                kind: TokenType::Word,
                value: Some("abc".to_string())
            })
        );
        assert_eq!(
            iter.next(),
            Some(&Token {
                kind: TokenType::TypeOf,
                value: None
            })
        );
        assert_eq!(
            iter.next(),
            Some(&Token {
                kind: TokenType::Word,
                value: Some("uint64_le".to_string())
            })
        );
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_basic_struct() {
        let tok = Tokeniser::new(
            "
        struct new_type {
            val1: type1,
            val2: type2
        }",
        );
        let mut iter = tok.iter();
        assert_eq!(
            iter.next(),
            Some(&Token {
                kind: TokenType::NewLine,
                value: None
            })
        );
        assert_eq!(
            iter.next(),
            Some(&Token {
                kind: TokenType::Word,
                value: Some("struct".to_string())
            })
        );
        assert_eq!(
            iter.next(),
            Some(&Token {
                kind: TokenType::Word,
                value: Some("new_type".to_string())
            })
        );
        assert_eq!(
            iter.next(),
            Some(&Token {
                kind: TokenType::OpenBrace,
                value: None
            })
        );
        assert_eq!(
            iter.next(),
            Some(&Token {
                kind: TokenType::NewLine,
                value: None
            })
        );
        assert_eq!(
            iter.next(),
            Some(&Token {
                kind: TokenType::Word,
                value: Some("val1".to_string())
            })
        );
        assert_eq!(
            iter.next(),
            Some(&Token {
                kind: TokenType::TypeOf,
                value: None
            })
        );
        assert_eq!(
            iter.next(),
            Some(&Token {
                kind: TokenType::Word,
                value: Some("type1".to_string())
            })
        );
        assert_eq!(
            iter.next(),
            Some(&Token {
                kind: TokenType::Comma,
                value: None
            })
        );
        assert_eq!(
            iter.next(),
            Some(&Token {
                kind: TokenType::NewLine,
                value: None
            })
        );
        assert_eq!(
            iter.next(),
            Some(&Token {
                kind: TokenType::Word,
                value: Some("val2".to_string())
            })
        );
        assert_eq!(
            iter.next(),
            Some(&Token {
                kind: TokenType::TypeOf,
                value: None
            })
        );
        assert_eq!(
            iter.next(),
            Some(&Token {
                kind: TokenType::Word,
                value: Some("type2".to_string())
            })
        );
        assert_eq!(
            iter.next(),
            Some(&Token {
                kind: TokenType::NewLine,
                value: None
            })
        );
        assert_eq!(
            iter.next(),
            Some(&Token {
                kind: TokenType::CloseBrace,
                value: None
            })
        );
    }
}
