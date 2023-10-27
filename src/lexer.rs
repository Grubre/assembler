use std::str::FromStr;

use thiserror::Error;

use crate::{
    specs::{Mnemonic, Register},
    token::{Token, TokenType},
};

#[derive(PartialEq, Eq, Debug, Error)]
pub enum LexerErr {
    #[error("Unknown token '{0}'.")]
    UnknownToken(String),
    #[error("Couldn't parse number '{0}'.")]
    NumberParseError(String),
}

// TODO: See if String can be used instead of [char], (possible utf-8 support(?))
pub struct Lexer<'a> {
    content: &'a [char],
    current_line: usize,
    current_char: usize,
}

impl<'a> Lexer<'a> {
    pub fn new(content: &'a [char]) -> Self {
        Self {
            content,
            current_line: 0,
            current_char: 0,
        }
    }

    fn peek(&self, offset: usize) -> Option<char> {
        if self.content.len() <= offset {
            return None;
        }
        Some(self.content[offset])
    }

    fn match_str(&self, pattern: String) -> bool {
        let pattern_len = pattern.len();
        if self.content.len() < pattern_len {
            return false;
        }

        // TODO: Find a better way to compare it
        pattern
            .chars()
            .zip(self.content.iter())
            .all(|(a, b)| a == *b)
    }

    fn chop(&mut self, len: usize) -> String {
        let lexeme = self.content[0..len].iter().collect();
        self.current_char += len;
        self.content = &self.content[len..];
        lexeme
    }

    fn chop_while<P>(&mut self, mut predicate: P) -> String
    where
        P: FnMut(&char) -> bool,
    {
        let mut i = 0;
        while i < self.content.len() && predicate(&self.content[i]) {
            i += 1;
        }
        self.chop(i)
    }

    fn trim_while<P>(&mut self, mut predicate: P)
    where
        P: FnMut(&char) -> bool,
    {
        while !self.content.is_empty() && predicate(&self.content[0]) {
            if self.content[0] == '\n' {
                self.current_char = 0;
            }
            self.current_char += 1;
            self.content = &self.content[1..]
        }
    }

    pub fn next_token(&mut self) -> Option<Result<Token, LexerErr>> {
        self.trim_while(|x| x.is_whitespace());

        if self.content.is_empty() {
            return None;
        }

        if self.content[0].is_ascii_digit() {
            let start = self.current_char;

            let (prefix, radix) = if self.match_str(String::from("0x")) {
                (self.chop(2), 16)
            } else if self.match_str(String::from("0b")) {
                (self.chop(2), 2)
            } else if self.match_str(String::from("0")) {
                (self.chop(1), 8)
            } else {
                (String::new(), 10)
            };

            let str = self.chop_while(|x| !x.is_whitespace());
            let number = i64::from_str_radix(&str, radix);

            let Ok(number) = number else {
                return Some(Err(LexerErr::NumberParseError(prefix + &str)));
            };

            return Some(Ok(Token::new(
                TokenType::Number(number),
                prefix + &str,
                self.current_line,
                start..self.current_char,
            )));
        }

        if self.content[0].is_alphabetic() {
            let start = self.current_char;
            let str = self.chop_while(|x| x.is_alphabetic());
            if let Ok(mnemonic) = Mnemonic::from_str(&str) {
                return Some(Ok(Token::new(
                    TokenType::Mnemonic(mnemonic),
                    str,
                    self.current_line,
                    start..self.current_char,
                )));
            }

            if let Ok(register) = Register::from_str(&str) {
                return Some(Ok(Token::new(
                    TokenType::Register(register),
                    str,
                    self.current_line,
                    start..self.current_char,
                )));
            }
        }

        Some(Err(LexerErr::UnknownToken(String::from(""))))
    }
}

impl<'a> Iterator for Lexer<'a> {
    type Item = Result<Token, LexerErr>;

    fn next(&mut self) -> Option<Self::Item> {
        self.next_token()
    }
}
