use std::str::FromStr;

use thiserror::Error;

use crate::{
    specs::{Mnemonic, Register},
    token::{Token, TokenType},
};

use phf::phf_map;

static KEYWORDS: phf::Map<&'static str, TokenType> = phf_map! {
    "byte" => TokenType::Byte,
};

// #TODO: Add number lines and character ranges to the error output
#[derive(PartialEq, Eq, Debug, Error)]
pub enum LexerErr {
    #[error("Unknown token '{0}'.")]
    UnknownToken(String),
    #[error("Couldn't parse number '{0}'.")]
    NumberParseError(String),
    #[error("Label '{0}:' should be at the beginning of the line.")]
    LabelParseError(String),
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
                self.current_line += 1;
            }
            // self.current_char += 1;
            self.content = &self.content[1..]
        }
    }

    fn parse_number(&mut self) -> Result<Token, LexerErr> {
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

        let str = self.chop_while(|x| x.is_ascii_hexdigit());
        let number = i64::from_str_radix(&str, radix);

        let Ok(number) = number else {
            return Err(LexerErr::NumberParseError(prefix + &str));
        };

        Ok(Token::new(
            TokenType::Number(number),
            prefix + &str,
            self.current_line,
            start..self.current_char,
        ))
    }

    fn parse_label(&mut self, start: usize, str: String) -> Result<Token, LexerErr> {
        self.chop(1);

        if start != 0 {
            return Err(LexerErr::LabelParseError(str));
        }

        Ok(Token::new(
            TokenType::Label(str.clone()),
            str,
            self.current_line,
            start..self.current_char,
        ))
    }

    pub fn next_token(&mut self) -> Option<Result<Token, LexerErr>> {
        self.trim_while(|x| x.is_whitespace());

        let start = self.current_char;

        if self.content.is_empty() {
            return None;
        }

        let initial_character = self.content[0];

        if self.content[0].is_ascii_digit() {
            let number = self.parse_number();
            return Some(number);
        }

        if self.content[0].is_alphabetic() {
            let str = self.chop_while(|x| x.is_alphabetic());

            if let Some(keyword) = KEYWORDS.get(&str).cloned() {
                return Some(Ok(Token::new(keyword, str, self.current_line, start..self.current_char)));
            }

            if let Some(':') = self.peek(0) {
                return Some(self.parse_label(start, str));
            }

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

        let character = match self.content[0] {
            '[' => Some((self.chop(1), TokenType::LeftSquareBracket)),
            ']' => Some((self.chop(1), TokenType::RightSquareBracket)),
            '#' => {
                self.chop(1);
                let str = self.chop_while(|x| x.is_alphanumeric());
                return Some(Ok(Token::new(
                    TokenType::LabelRef(str.clone()),
                    str,
                    self.current_line,
                    start..self.current_char,
                )));
            }
            _ => None,
        };

        if let Some((str, token_type)) = character {
            return Some(Ok(Token::new(
                token_type,
                str,
                self.current_line,
                self.current_char - 1..self.current_char,
            )));
        };

        Some(Err(LexerErr::UnknownToken(String::from(initial_character))))
    }
}

impl<'a> Iterator for Lexer<'a> {
    type Item = Result<Token, LexerErr>;

    fn next(&mut self) -> Option<Self::Item> {
        self.next_token()
    }
}
