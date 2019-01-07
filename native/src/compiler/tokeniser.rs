#[derive(PartialEq, Debug, Clone)]
pub struct Token {
    pub kind: TokenType,
    value: Option<String>,
}

impl Token {
    pub fn new(kind: TokenType, value: Option<String>) -> Token {
        Token { kind, value }
    }

    pub fn get_value(&self) -> &str {
        if let Some(val) = &self.value {
            &val
        } else {
            panic!("Expected value from token: {:?}", self);
        }
    }
}

#[derive(PartialEq, Debug, Clone)]
pub enum TokenType {
    Word,
    TypeOf,
    NewLine,
    OpenBrace,
    CloseBrace,
    Comma,
}

pub struct Tokeniser {
    tokens: Vec<Token>,
}

impl Tokeniser {
    pub fn new(data: &str) -> Tokeniser {
        let mut tokens: Vec<Token> = Vec::new();
        let mut state: Box<dyn TokeniserState> = Box::new(EmptyState {});
        for c in data.chars() {
            state = state.new_char(c, &mut tokens);
        }
        if let Some(t) = state.get_token() {
            tokens.push(t);
        }
        Tokeniser { tokens }
    }

    pub fn iter<'a>(&'a self) -> IterTokeniser<'a> {
        IterTokeniser {
            inner: self,
            pos: 0,
        }
    }
}

pub struct IterTokeniser<'a> {
    inner: &'a Tokeniser,
    pos: usize,
}

impl<'a> Iterator for IterTokeniser<'a> {
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

trait TokeniserState {
    fn new_char(self: Box<Self>, c: char, tokens: &mut Vec<Token>) -> Box<dyn TokeniserState>;
    fn get_token(self: Box<Self>) -> Option<Token>;
}

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
        if c.is_alphabetic() || c.is_numeric() || c == '_' {
            self.value.push(c);
            return self;
        } else {
            if let Some(t) = self.get_token() {
                tokens.push(t);
            } else {
                panic!("Expected token");
            }

            if let Some(s) = new_state(c, tokens) {
                return s;
            }
        }
        Box::new(EmptyState {})
    }

    fn get_token(self: Box<Self>) -> Option<Token> {
        Some(Token::new(TokenType::Word, Some(self.value)))
    }
}

fn new_state(c: char, tokens: &mut Vec<Token>) -> Option<Box<dyn TokeniserState>> {
    if c == '\n' {
        tokens.push(Token::new(TokenType::NewLine, None));
        return None;
    }
    if c.is_whitespace() {
        return None;
    }
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
}
