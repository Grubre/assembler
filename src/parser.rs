use thiserror::Error;

use crate::token::{Token, TokenType};

// TODO: Add spans and line numbers to errors
#[derive(PartialEq, Eq, Debug, Error)]
pub enum ParserErr {
    #[error("Expected: \"{0}\", found \"{1}\".")]
    UnexpectedToken(String, String),
    #[error("Line should begin with a Mnemonic, 'byte' or a label, instead found \"{0}\".")]
    UnexpectedLineBeginning(String),
    #[error("Expected: \"{0}\", instead hit EOF.")]
    EOF(String),
}

// TODO: Make the tokens be an iterator, maybe(?)
struct Parser<'a> {
    tokens: &'a [Token],
}

// FIXME: operands is a vec of tokens, because of that you can't differentiate
//        between a number and a memref.
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

    fn peek(&self) -> Option<&Token> {
        if self.tokens.is_empty() {
            return None;
        }
        let token = &self.tokens[0];
        Some(token)
    }

    fn parse(&mut self) -> Result<Vec<Line>, Vec<ParserErr>> {
        let mut lines = vec![];
        let mut errors = vec![];

        let mut error_recovery = false;
        while !self.tokens.is_empty() {
            let Some(token) = &self.peek() else {
                break;
            };
            match token.token_type {
                TokenType::Mnemonic(_) => {
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
                        errors.push(ParserErr::UnexpectedLineBeginning(token.content.clone()));
                        error_recovery = false;
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

    fn byte(&mut self) -> Result<Line, ParserErr> {
        let _byte = self.chop().unwrap();

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
        let token = self.chop().ok_or(ParserErr::EOF("Number".to_string()))?;
        match token.token_type {
            TokenType::Number(_) => {}
            _ => {
                return Err(ParserErr::UnexpectedToken(
                    "Number".to_string(),
                    token.content,
                ))
            }
        };
        Ok(token)
    }

    fn register(&mut self) -> Result<Token, ParserErr> {
        let token = self.chop().ok_or(ParserErr::EOF("Register".to_string()))?;
        match token.token_type {
            TokenType::Register(_) => {}
            _ => {
                return Err(ParserErr::UnexpectedToken(
                    "Register".to_string(),
                    token.content,
                ))
            }
        };
        Ok(token)
    }

    fn labelref(&mut self) -> Result<Token, ParserErr> {
        let token = self.chop().ok_or(ParserErr::EOF("LabelRef".to_string()))?;
        match token.token_type {
            TokenType::LabelRef(_) => {}
            _ => {
                return Err(ParserErr::UnexpectedToken(
                    "LabelRef".to_string(),
                    token.content,
                ))
            }
        };
        Ok(token)
    }

    fn operand(&mut self) -> Option<Result<Token, ParserErr>> {
        let token = self.peek()?;
        match token.token_type {
            TokenType::Register(_) => Some(self.register()),
            TokenType::Number(_) => Some(self.number()),
            TokenType::LabelRef(_) => Some(self.labelref()),
            TokenType::LeftSquareBracket => Some(self.memref()),
            _ => None,
        }
    }

    fn memref(&mut self) -> Result<Token, ParserErr> {
        let _left_bracket = self.chop().ok_or(ParserErr::EOF("'['".to_string()))?; // chops the '['

        let token = self
            .chop()
            .ok_or(ParserErr::EOF("Number or LabelRef".to_string()))?;
        match token.token_type {
            TokenType::Number(_) | TokenType::LabelRef(_) => {}
            _ => {
                return Err(ParserErr::UnexpectedToken(
                    "Number or LabelRef".to_string(),
                    token.content,
                ))
            }
        };

        let right_bracket = self.chop().ok_or(ParserErr::EOF("']'".to_string()))?;
        match right_bracket.token_type {
            TokenType::RightSquareBracket => Ok(token),
            _ => Err(ParserErr::UnexpectedToken("']".to_string(), token.content)),
        }
    }
}

pub fn parse(tokens: &[Token]) -> Result<Vec<Line>, Vec<ParserErr>> {
    let mut parser = Parser::new(tokens);
    parser.parse()
}
