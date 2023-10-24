use thiserror::Error;

use crate::{
    error::*,
    token::Token,
};

#[derive(PartialEq, Eq, Debug, Error)]
pub enum LexerErr {
    #[error("Unknown token")]
    UnknownToken,
}

pub struct Lexer<'a> {
    content: &'a [char],
    current_line: u32,
    current_char: u32,
}

impl<'a> Lexer<'a> {
    pub fn new(content: &'a [char]) -> Self {
        Self { content, current_line: 0, current_char: 0 }
    }

    fn trim_while<P>(&mut self, mut predicate: P)
    where
        P: FnMut(&char) -> bool,
    {
        while !self.content.is_empty() && predicate(&self.content[0]) {
            self.current_char += 1;
            if self.content[0] == '\n' {
                self.current_char = 0;
                self.current_char += 1;
            }
            self.content = &self.content[1..]
        }
    }

    pub fn next_token(&mut self) -> Option<Result<Token, Error>> {
        self.trim_while(|x| x.is_whitespace());

        if self.content.is_empty() {
            return None;
        }

        match self.content[0] {

        }
    }
}
