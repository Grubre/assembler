use thiserror::Error;

use crate::token::{Token, TokenType};

// TODO: Add spans and line numbers to errors
#[derive(PartialEq, Eq, Debug, Error)]
pub enum ParserErr {
    #[error("a")]
    TempVal,
    #[error("Expected: \"{0}\", found \"{1}\".")]
    UnexpectedToken(String, String),
}

// TODO: Make the tokens be an iterator
struct Parser<'a> {
    tokens: &'a [Token],
}

#[derive(Debug)]
pub enum Line {
    Byte(Vec<Token>),
    Instruction {
        mnemonic: Token,
        operands: Vec<Token>,
    },
}

/*
Grammar:
line -> instruction | byte;

label -> STRING ":";
instruction -> mnemonic (operand)*;
byte -> "byte" (NUMBER)+;
*/

impl<'a> Parser<'a> {
    fn new(tokens: &'a [Token]) -> Self {
        Self { tokens }
    }

    fn chop(&mut self) -> Option<Token> {
        if self.tokens.is_empty() {
            return None;
        }
        let token = &self.tokens[0];
        self.tokens = &self.tokens[1..];
        // TODO: remove the clone
        Some(token.clone())
    }

    fn peek(&mut self) -> Option<&Token> {
        if self.tokens.is_empty() {
            return None;
        }
        let token = &self.tokens[0];
        Some(token)
    }

    fn parse(&mut self) -> Result<Vec<Line>, Vec<ParserErr>> {
        let mut lines = vec![];
        let mut errors = vec![];
        for token in self.tokens {
            let token_type = &token.token_type;
            match token_type {
                TokenType::Mnemonic(_) => {
                    let line = self.line();
                    if let Ok(line) = line {
                        lines.push(line);
                    };
                }
                TokenType::Byte => {
                    let byte = self.byte();
                    if let Ok(byte) = byte {
                        lines.push(byte);
                    };
                }
                TokenType::Label(_) => continue,
                _ => errors.push(ParserErr::TempVal),
            }
            dbg!(token);
        }

        if errors.is_empty() {
            return Ok(lines);
        }
        Err(errors)
    }

    fn line(&mut self) -> Result<Line, ParserErr> {
        let token = self.peek().ok_or(ParserErr::TempVal)?;
        match token.token_type {
            TokenType::Mnemonic(_) => self.instruction(),
            TokenType::Byte => self.byte(),
            // TODO: Add proper error here
            _ => Err(ParserErr::TempVal),
        }
    }

    // TODO:
    fn byte(&mut self) -> Result<Line, ParserErr> {
        let mut numbers = vec![];
        while let Some(token) = self.peek() {
            match token.token_type {
                TokenType::Number(_) => numbers.push(self.number()?),
                _ => break,
            }
        }
        Ok(Line::Byte(numbers))
    }

    fn instruction(&mut self) -> Result<Line, ParserErr> {
        let mnemonic = self.chop().unwrap();
        let mut operands = vec![];

        while let Some(token) = self.operand() {
            match token {
                Ok(token) => operands.push(token),
                Err(err) => return Err(err),
            }
        }

        Ok(Line::Instruction { mnemonic, operands })
    }

    fn number(&mut self) -> Result<Token, ParserErr> {
        let token = self.chop().ok_or(ParserErr::TempVal)?;
        match token.token_type {
            TokenType::Number(_) => {}
            _ => return Err(ParserErr::TempVal), // TODO: Found unexpected token
        };
        Ok(token)
    }

    fn register(&mut self) -> Result<Token, ParserErr> {
        let token = self.chop().ok_or(ParserErr::TempVal)?;
        match token.token_type {
            TokenType::Register(_) => {}
            _ => return Err(ParserErr::TempVal), // TODO: Found unexpected token
        };
        Ok(token)
    }

    fn operand(&mut self) -> Option<Result<Token, ParserErr>> {
        let token = self.peek()?;
        match token.token_type {
            TokenType::Register(_) => Some(self.register()),
            TokenType::Number(_) => Some(self.number()),
            TokenType::LabelRef(_) => todo!(),
            TokenType::LeftSquareBracket => Some(self.memref()),
            _ => None,
        }
    }

    fn memref(&mut self) -> Result<Token, ParserErr> {
        let _left_bracket = self.chop().ok_or(ParserErr::TempVal)?; // chops the '['

        let token = self.chop().ok_or(ParserErr::TempVal)?;
        match token.token_type {
            TokenType::Number(_) | TokenType::LabelRef(_) => {}
            _ => return Err(ParserErr::TempVal), // TODO: Found unexpected token
        };

        let right_bracket = self.chop().ok_or(ParserErr::TempVal)?;
        match right_bracket.token_type {
            TokenType::RightSquareBracket => Ok(token),
            _ => Err(ParserErr::TempVal), // TODO: Found unexpected token
        }
    }
}

pub fn parse(tokens: &[Token]) -> Result<Vec<Line>, Vec<ParserErr>> {
    let mut parser = Parser::new(tokens);
    parser.parse()
}
