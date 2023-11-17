use thiserror::Error;

use crate::{
    specs::Operand,
    token::{Token, TokenType},
};

// TODO: Add spans and line numbers to errors
#[derive(PartialEq, Eq, Debug, Error)]
pub enum ParserErr<'a> {
    #[error("Expected: \"{0}\", found \"{1}\".")]
    UnexpectedToken(&'a str, &'a str),
    #[error("Line should begin with a Mnemonic, 'byte' or a label, instead found \"{0}\".")]
    UnexpectedLineBeginning(&'a str),
    #[error("Expected: \"{0}\", instead hit EOF.")]
    EOF(String),
}

struct Parser<'a> {
    tokens: &'a [Token],
}

#[derive(Debug)]
pub enum Line<'a> {
    Byte(Vec<&'a Token>),
    Instruction {
        mnemonic: &'a Token,
        operands: Vec<(Operand, &'a Token)>,
    },
}

/*
Grammar:
line -> (label)? instruction | byte;

label -> STRING ":";

instruction -> mnemonic (operand)*;
byte -> "byte" (NUMBER)+;

operand -> register | NUMBER | labelref | memref;
register -> "A" | "B" | "F";
labelref -> '#' STRING;
memref -> '[' (labelref | NUMBER) ']';*/

impl<'a> Parser<'a> {
    fn new(tokens: &'a [Token]) -> Self {
        Self { tokens }
    }

    fn chop(&mut self) -> Option<&'a Token> {
        let token = self.tokens.get(0)?;
        self.tokens = &self.tokens[1..];
        Some(token)
    }

    fn peek(&self) -> Option<&'a Token> {
        if self.tokens.is_empty() {
            return None;
        }
        let token = &self.tokens[0];
        Some(token)
    }

    fn parse(&mut self) -> Result<Vec<Line<'a>>, Vec<ParserErr<'a>>> {
        let mut lines = vec![];
        let mut errors = vec![];

        let mut error_recovery = false;
        while !self.tokens.is_empty() {
            let Some(token) = &self.peek() else {
                break;
            };
            match token.token_type {
                TokenType::Mnemonic(_) => {
                    error_recovery = false;
                    let line = self.instruction();
                    match line {
                        Ok(line) => lines.push(line),
                        Err(err) => {
                            error_recovery = true;
                            errors.push(err)
                        }
                    };
                }
                TokenType::Byte => {
                    error_recovery = false;
                    let line = self.byte();
                    match line {
                        Ok(line) => lines.push(line),
                        Err(err) => {
                            error_recovery = true;
                            errors.push(err)
                        }
                    };
                }
                TokenType::Label(_) => {
                    self.chop();
                }
                _ => {
                    if !error_recovery {
                        errors.push(ParserErr::UnexpectedLineBeginning(&token.content));
                    }
                    self.chop();
                }
            }
        }

        if errors.is_empty() {
            return Ok(lines);
        }
        Err(errors)
    }

    fn byte(&mut self) -> Result<Line<'a>, ParserErr<'a>> {
        let _byte = self.chop().unwrap();

        let mut numbers = vec![];
        while let Some(token) = self.peek() {
            match token.token_type {
                TokenType::Number(_) => numbers.push(self.number()?.1),
                _ => break,
            }
        }
        Ok(Line::Byte(numbers))
    }

    fn instruction(&mut self) -> Result<Line<'a>, ParserErr<'a>> {
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

    // TODO: Remove code duplication for these three functions
    fn number(&mut self) -> Result<(Operand, &'a Token), ParserErr<'a>> {
        let token = self.chop().ok_or(ParserErr::EOF("Number".to_string()))?;
        match token.token_type {
            TokenType::Number(_) => {}
            _ => {
                return Err(ParserErr::UnexpectedToken(
                    &"Number",
                    &token.content,
                ))
            }
        };
        Ok((Operand::Const, token))
    }

    fn register(&mut self) -> Result<(Operand, &'a Token), ParserErr<'a>> {
        let token = self.chop().ok_or(ParserErr::EOF("Register".to_string()))?;
        let reg = match &token.token_type {
            TokenType::Register(reg) => reg.clone(),
            _ => {
                return Err(ParserErr::UnexpectedToken(
                    "Register",
                    &token.content,
                ))
            }
        };
        Ok((Operand::Register(reg), token))
    }

    fn labelref(&mut self) -> Result<(Operand, &'a Token), ParserErr<'a>> {
        let token = self.chop().ok_or(ParserErr::EOF("LabelRef".to_string()))?;
        match token.token_type {
            TokenType::LabelRef(_) => {}
            _ => {
                return Err(ParserErr::UnexpectedToken(
                    "LabelRef",
                    &token.content,
                ))
            }
        };
        Ok((Operand::Const, token))
    }

    fn memref(&mut self) -> Result<(Operand, &'a Token), ParserErr<'a>> {
        let _left_bracket = self.chop().ok_or(ParserErr::EOF("[".to_string()))?; // chops the '['

        let token = self
            .chop()
            .ok_or(ParserErr::EOF("Number or LabelRef".to_string()))?;
        match token.token_type {
            TokenType::Number(_) | TokenType::LabelRef(_) => {}
            _ => {
                return Err(ParserErr::UnexpectedToken(
                    "Number or LabelRef",
                    &token.content,
                ))
            }
        };

        let right_bracket = self.chop().ok_or(ParserErr::EOF("]".to_string()))?;
        match right_bracket.token_type {
            TokenType::RightSquareBracket => Ok((Operand::Mem, token)),
            _ => Err(ParserErr::UnexpectedToken(
                "]",
                &right_bracket.content,
            )),
        }
    }

    fn operand(&mut self) -> Option<Result<(Operand, &'a Token), ParserErr<'a>>> {
        let token = self.peek()?;
        match token.token_type {
            TokenType::Register(_) => Some(self.register()),
            TokenType::Number(_) => Some(self.number()),
            TokenType::LabelRef(_) => Some(self.labelref()),
            TokenType::LeftSquareBracket => Some(self.memref()),
            _ => None,
        }
    }
}

pub fn parse<'a>(tokens: &'a [Token]) -> Result<Vec<Line<'a>>, Vec<ParserErr<'a>>> {
    let mut parser = Parser::new(tokens);
    parser.parse()
}
