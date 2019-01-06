use std::fs;

#[derive(PartialEq, Debug, Clone)]
pub struct Token {
    kind: TokenType,
    value: Option<String>,
}

#[derive(PartialEq, Debug, Clone)]
pub enum TokenType {
    Word,
    TypeOf,
}

pub struct Tokeniser {
    tokens: Vec<Token>,
    curr_pos: usize,
}

impl Tokeniser {
    fn new(data: String) -> Tokeniser {
        let mut tokens: Vec<Token> = Vec::new();
        let mut state: Box<dyn State> = Box::new(EmptyState {});
        for c in data.chars() {
            state = state.new_char(c, &mut tokens);
        }
        if let Some(t) = state.get_token() {
            tokens.push(t);
        }
        Tokeniser {
            tokens,
            curr_pos: 0,
        }
    }
}

impl Iterator for Tokeniser {
    type Item = Token;

    fn next(&mut self) -> Option<Token> {
        if self.curr_pos < self.tokens.len() {
            let t = self.tokens[self.curr_pos].clone();
            self.curr_pos += 1;
            Some(t)
        } else {
            None
        }
    }
}

pub fn load_file(filename: &str) -> Tokeniser {
    let s = fs::read_to_string(filename).unwrap();
    Tokeniser::new(s)
}

trait State {
    fn new_char(self: Box<Self>, c: char, tokens: &mut Vec<Token>) -> Box<dyn State>;
    fn get_token(self: Box<Self>) -> Option<Token>;
}

struct EmptyState;

impl State for EmptyState {
    fn new_char(self: Box<Self>, c: char, tokens: &mut Vec<Token>) -> Box<dyn State> {
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

impl State for WordState {
    fn new_char(mut self: Box<Self>, c: char, tokens: &mut Vec<Token>) -> Box<dyn State> {
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
        Some(Token {
            kind: TokenType::Word,
            value: Some(self.value),
        })
    }
}

fn new_state(c: char, tokens: &mut Vec<Token>) -> Option<Box<dyn State>> {
    if c.is_whitespace() {
        return None;
    }
    if c.is_alphabetic() || c == '_' {
        return Some(Box::new(WordState::new(c)));
    }

    // No state needed
    if c == ':' {
        tokens.push(Token { kind: TokenType::TypeOf, value: None });
        return None;
    }

    panic!("Unknown symbol: {}", c);
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_load_file() {
        let _ = load_file("test_load_file.carta");
    }

    #[test]
    fn test_tokenise_word() {
        let mut tok = Tokeniser::new("abc".to_string());
        assert_eq!(
            tok.next(),
            Some(Token {
                kind: TokenType::Word,
                value: Some("abc".to_string())
            })
        );
        assert_eq!(tok.next(), None);
    }

    #[test]
    fn test_space_at_start_plus_number() {
        let mut tok = Tokeniser::new(" abc23".to_string());
        assert_eq!(
            tok.next(),
            Some(Token {
                kind: TokenType::Word,
                value: Some("abc23".to_string())
            })
        );
        assert_eq!(tok.next(), None);
    }

    #[test]
    fn test_newline_at_start_plus_underscore() {
        let mut tok = Tokeniser::new("\n_abc_abc".to_string());
        assert_eq!(
            tok.next(),
            Some(Token {
                kind: TokenType::Word,
                value: Some("_abc_abc".to_string())
            })
        );
        assert_eq!(tok.next(), None);
    }

    #[test]
    fn test_whitespace_at_start_tab() {
        let mut tok = Tokeniser::new("\tabc".to_string());
        assert_eq!(
            tok.next(),
            Some(Token {
                kind: TokenType::Word,
                value: Some("abc".to_string())
            })
        );
        assert_eq!(tok.next(), None);
    }

    #[test]
    fn test_multiple_words() {
        let mut tok = Tokeniser::new("abc def\nghi\tjkl ".to_string());
        assert_eq!(
            tok.next(),
            Some(Token {
                kind: TokenType::Word,
                value: Some("abc".to_string())
            })
        );
        assert_eq!(
            tok.next(),
            Some(Token {
                kind: TokenType::Word,
                value: Some("def".to_string())
            })
        );
        assert_eq!(
            tok.next(),
            Some(Token {
                kind: TokenType::Word,
                value: Some("ghi".to_string())
            })
        );
        assert_eq!(
            tok.next(),
            Some(Token {
                kind: TokenType::Word,
                value: Some("jkl".to_string())
            })
        );
        assert_eq!(tok.next(), None);
    }

    #[test]
    fn test_basic_typeof() {
        let mut tok = Tokeniser::new("abc: uint64_le".to_string());
        assert_eq!(
            tok.next(),
            Some(Token {
                kind: TokenType::Word,
                value: Some("abc".to_string())
            })
        );
        assert_eq!(
            tok.next(),
            Some(Token {
                kind: TokenType::TypeOf,
                value: None
            })
        );
        assert_eq!(
            tok.next(),
            Some(Token {
                kind: TokenType::Word,
                value: Some("uint64_le".to_string())
            })
        );
        assert_eq!(tok.next(), None);
    }
}
