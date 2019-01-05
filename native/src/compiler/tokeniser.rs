use std::fs;

use std::cmp::PartialEq;
use std::iter::IntoIterator;
use std::string::String;

use regex::Regex;

#[derive(PartialEq, Debug)]
pub struct Token {
    kind: TokenType,
    value: String,
}

#[derive(PartialEq, Debug)]
pub enum TokenType {
    WORD,
}

pub struct Tokeniser {
    file_data: std::str::Chars,
    curr_pos: usize,
}

impl Tokeniser {
    fn new(mut data: String) -> Tokeniser {
        let i = IntoIterator::into_iter(data.chars());
        Tokeniser {
            file_data: i,
            curr_pos: 0,
        }
    }

    pub fn next(&mut self) -> Token {
        lazy_static! {
            static ref RE_WORD: Regex = Regex::new(r"^([a-zA-Z_][a-zA-Z_0-9]*)").unwrap();
        }

        /*while self.curr_pos < self.file_data.len()
            && (self.file_data[self.curr_pos] == ' '
                || self.file_data[self.curr_pos] == '\n'
                || self.file_data[self.curr_pos] == '\t')
        {
            self.curr_pos += 1;
        }*/

        //if let Some(cap) = RE_WORD.captures(self.file_data[self.curr_pos..]) {}

        Token {
            kind: TokenType::WORD,
            value: "???".to_string(),
        }
    }
}

pub fn load_file(filename: &str) -> Tokeniser {
    let s = fs::read_to_string(filename).unwrap();
    Tokeniser::new(s)
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
            Token {
                kind: TokenType::WORD,
                value: "abc".to_string()
            }
        );
    }

    #[test]
    fn test_whitespace_at_start_space() {
        let mut tok = Tokeniser::new(" abc".to_string());
        assert_eq!(
            tok.next(),
            Token {
                kind: TokenType::WORD,
                value: "abc".to_string()
            }
        );
    }

    #[test]
    fn test_whitespace_at_start_newline() {
        let mut tok = Tokeniser::new("\nabc".to_string());
        assert_eq!(
            tok.next(),
            Token {
                kind: TokenType::WORD,
                value: "abc".to_string()
            }
        );
    }

    #[test]
    fn test_whitespace_at_start_tab() {
        let mut tok = Tokeniser::new("\tabc".to_string());
        assert_eq!(
            tok.next(),
            Token {
                kind: TokenType::WORD,
                value: "abc".to_string()
            }
        );
    }
}
