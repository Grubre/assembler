use thiserror::Error;

use crate::{
    config::{ArgDef, Config, InstructionDef},
    error::{Error, ResultSplit, WithSpan}, token::{Span, Token, TokenType},
};
use std::{collections::HashMap, num::ParseIntError};

#[derive(Debug, Error)]
pub enum ParseErr {
    #[error("Can't find mapping")]
    NoMapping,
    #[error("Isn't an argument for instruction")]
    NotAnArg,
    #[error("Line has to start with a mnemonic or a byte declaration")]
    WrongFirst,
    #[error("Not a number: {0}")]
    NotANumber(#[from] ParseIntError),
    #[error("EmptyLine")]
    EmptyLine,
}

#[derive(Debug)]
pub enum Register {
    A,
    B,
    F,
}

#[derive(Debug)]
pub enum Value {
    Num(i8),
    LabelRef(String),
}

#[derive(Debug)]
pub enum ArgKind {
    Register(Register),
    ImmediateValue(Value),
    MemAddress(Value),
}

impl ArgKind {
    pub fn with_span(self, span: Span) -> Arg {
        Arg { kind: self, span }
    }
}

#[derive(Debug)]
pub struct Arg {
    kind: ArgKind,
    span: Span,
}

#[derive(Debug)]
pub struct Instruction {
    mnem: String,
    span: Span,
    args: Vec<Arg>,
}

#[derive(Debug)]
pub struct Labels(pub HashMap<String, usize>);

///MATCHING

impl Arg {
    pub fn matches_def(&self, def: &ArgDef) -> bool {
        match self.kind {
            ArgKind::Register(Register::A) => *def == ArgDef::A,
            ArgKind::Register(Register::B) => *def == ArgDef::B,
            ArgKind::Register(Register::F) => *def == ArgDef::F,
            ArgKind::ImmediateValue(_) => *def == ArgDef::Const,
            ArgKind::MemAddress(_) => *def == ArgDef::Mem,
        }
    }
}

impl Instruction {
    fn matches_def(&self, def: &InstructionDef) -> bool {
        self.mnem == def.mnem
            && self.args.len() == def.args_def.len()
            && self
                .args
                .iter()
                .zip(def.args_def.iter())
                .fold(true, |acc, (arg, def)| acc && arg.matches_def(def))
    }

    fn find_match<'a>(&self, config: &'a Config) -> Option<&'a InstructionDef> {
        config.0.iter().find(|&def| self.matches_def(def))
    }
}

pub fn parse_number(str: &str) -> Result<i8, ParseErr> {
    let is_negative = str.starts_with('-');
    let str = if is_negative { &str[1..] } else { str };

    let result = if let Some(num) = str.strip_prefix("0x") {
        i8::from_str_radix(num, 16)
    } else if let Some(num) = str.strip_prefix("0b") {
        i8::from_str_radix(num, 2)
    } else if let Some(num) = str.strip_prefix("0o") {
        i8::from_str_radix(num, 8)
    } else {
        str.parse::<i8>()
    }
    .map_err(|err| err.into());

    match result {
        Ok(num) if is_negative => Ok(-num),
        _ => result,
    }
}

impl TryFrom<Token> for Arg {
    type Error = Error;

    fn try_from(token: Token) -> Result<Self, Self::Error> {
        match &token.token_type {
            TokenType::Register => match token.content.as_str() {
                "A" => Ok(ArgKind::Register(Register::A)),
                "B" => Ok(ArgKind::Register(Register::B)),
                "F" => Ok(ArgKind::Register(Register::F)),
                _ => unreachable!(),
            },
            TokenType::Number => Ok(ArgKind::ImmediateValue(Value::Num(
                parse_number(&token.content).map_err(|err| err.with_span(token.span.clone()))?,
            ))),
            TokenType::LabelRef => Ok(ArgKind::ImmediateValue(Value::LabelRef(
                token.content.clone(),
            ))),
            TokenType::MemAddress => Ok(ArgKind::MemAddress(Value::Num(
                parse_number(&token.content).map_err(|err| err.with_span(token.span.clone()))?,
            ))),
            TokenType::LabelAddressRef => {
                Ok(ArgKind::MemAddress(Value::LabelRef(token.content.clone())))
            }
            _ => Err(ParseErr::NotAnArg.with_span(token.span.clone())),
        }
        .map(|kind| kind.with_span(token.span))
    }
}

#[derive(Debug)]
pub enum Unresolved {
    LabelRef(String, Span),
    Value(String),
}

impl Arg {
    pub fn to_unresolved_binary(self) -> Option<Unresolved> {
        match self.kind {
            ArgKind::Register(_) => None,
            ArgKind::ImmediateValue(Value::Num(num)) => {
                Some(Unresolved::Value(format!("{:08b}", num)))
            }
            ArgKind::MemAddress(Value::Num(num)) => Some(Unresolved::Value(format!("{:08b}", num))),
            ArgKind::ImmediateValue(Value::LabelRef(label)) => {
                Some(Unresolved::LabelRef(label, self.span))
            }
            ArgKind::MemAddress(Value::LabelRef(label)) => {
                Some(Unresolved::LabelRef(label, self.span))
            }
        }
    }
}

fn parse_line(
    labels: &mut HashMap<String, usize>,
    curr_line: &mut usize,
    tokens: Vec<Token>,
    config: &Config,
) -> Result<Vec<Unresolved>, Vec<Error>> {
    let line_span = tokens
        .iter()
        .map(|tok| tok.span.clone())
        .fold(Span::new(0, 0..0), |a, b| a + b);

    let mut iter = tokens.into_iter().skip_while(|token| {
        if token.token_type == TokenType::Label {
            labels.insert(token.content.to_owned(), *curr_line);
            true
        } else {
            false
        }
    });

    let first_token = iter
        .next()
        .ok_or(vec![ParseErr::EmptyLine.with_span(line_span.clone())])?;
    let mut ret = Vec::new();

    match first_token.token_type {
        TokenType::Mnemonic => {
            let content = first_token.content;

            let args = iter.map(|token| token.try_into()).result_split()?;

            let instr = Instruction {
                mnem: content,
                args,
                span: line_span,
            };

            let def = instr
                .find_match(config)
                .ok_or(vec![ParseErr::NoMapping.with_span(instr.span)])?;

            ret.push(Unresolved::Value(def.binary.clone()));

            for arg in instr.args {
                if let Some(unres) = arg.to_unresolved_binary() {
                    ret.push(unres);
                }
            }
        }
        TokenType::Byte => {
            for tok in iter {
                ret.push(Unresolved::Value(format!(
                    "{:08b}",
                    parse_number(&tok.content)
                        .map_err(|err| vec![err.with_span(tok.span.clone())])?
                )));
            }
        }
        _ => {
            return Err(vec![ParseErr::WrongFirst.with_span(first_token.span)]);
        }
    };
    *curr_line += ret.len();

    Ok(ret)
}

pub fn parse_all(
    lines: Vec<Vec<Token>>,
    config: &Config,
) -> Result<(Vec<Unresolved>, Labels), Vec<Error>> {
    let mut unresolved = Vec::new();
    let mut errors = Vec::new();

    let mut labels = Labels(HashMap::new());

    let mut curr_line = 0;
    for line in lines {
        let ret = parse_line(&mut labels.0, &mut curr_line, line, config);
        match ret {
            Ok(mut unr) => unresolved.append(&mut unr),
            Err(mut err) => errors.append(&mut err),
        }
    }

    if errors.is_empty() {
        Ok((unresolved, labels))
    } else {
        Err(errors)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test() {
        let dummy_span = Span::new(0, 0..0);

        let instr = Instruction {
            mnem: "SUB".to_string(),
            args: vec![
                ArgKind::MemAddress(Value::Num(5)).with_span(dummy_span.clone()),
                ArgKind::Register(Register::A).with_span(dummy_span.clone()),
                ArgKind::Register(Register::B).with_span(dummy_span.clone()),
            ],
            span: dummy_span,
        };
        let def = InstructionDef {
            mnem: "SUB".to_string(),
            args_def: vec![ArgDef::Mem, ArgDef::A, ArgDef::B],
            mnem_full: "SUBABMEM".to_string(),
            binary: "c0011011".to_string(),
        };

        assert!(instr.matches_def(&def));
    }

    #[test]
    fn t_test() {
        assert!(ArgKind::MemAddress(Value::Num(5))
            .with_span(Span::new(0, 0..0))
            .matches_def(&ArgDef::Mem));
    }
    use super::parse_number;

    #[test]
    fn test_parse_number_decimal() {
        assert_eq!(parse_number("42").unwrap(), 42);
        assert_eq!(parse_number("-42").unwrap(), -42);
        assert_eq!(parse_number("0").unwrap(), 0);
    }

    #[test]
    fn test_parse_number_hexadecimal() {
        assert_eq!(parse_number("0x2A").unwrap(), 42);
        assert_eq!(parse_number("0x2a").unwrap(), 42);
        assert_eq!(parse_number("-0x2A").unwrap(), -42);
        assert_eq!(parse_number("0x0").unwrap(), 0);
    }

    #[test]
    fn test_parse_number_binary() {
        assert_eq!(parse_number("0b101010").unwrap(), 42);
        assert_eq!(parse_number("0b0").unwrap(), 0);
    }

    #[test]
    fn test_parse_number_octal() {
        assert_eq!(parse_number("0o52").unwrap(), 42);
        assert_eq!(parse_number("-0o52").unwrap(), -42);
        assert_eq!(parse_number("0o0").unwrap(), 0);
    }

    #[test]
    fn test_parse_number_invalid_input() {
        assert!(parse_number("invalid").is_err());
        assert!(parse_number("0xG").is_err());
        assert!(parse_number("0b3").is_err());
        assert!(parse_number("0o9").is_err());
    }

    #[test]
    fn test_parse_number_overflow() {
        assert!(parse_number("9223372036854775808").is_err());
        assert!(parse_number("-9223372036854775809").is_err());
        assert!(parse_number("0x8000000000000000").is_err());
        assert!(
            parse_number("0b1000000000000000000000000000000000000000000000000000000000000000")
                .is_err()
        );
        assert!(parse_number("0o1000000000000000000000").is_err());
    }

    #[test]
    fn test_instruction_find_match() {
        let dummy_span = Span::new(0, 0..0);
        let config = Config(vec![InstructionDef {
            mnem: "SUB".to_string(),
            args_def: vec![ArgDef::Mem, ArgDef::A, ArgDef::B],
            mnem_full: "SUBABMEM".to_string(),
            binary: "c0011011".to_string(),
        }]);

        let instr = Instruction {
            mnem: "SUB".to_string(),
            args: vec![
                ArgKind::MemAddress(Value::Num(5)).with_span(dummy_span.clone()),
                ArgKind::Register(Register::A).with_span(dummy_span.clone()),
                ArgKind::Register(Register::B).with_span(dummy_span.clone()),
            ],
            span: dummy_span,
        };

        assert!(instr.find_match(&config).is_some());
    }

    #[test]
    fn test_arg_try_from_invalid_token() {
        let invalid_token = Token {
            token_type: TokenType::Label,
            content: "label".to_string(),
            span: Span::new(0, 0..5),
        };

        assert!(Arg::try_from(invalid_token).is_err());
    }

    #[test]
    fn test_parse_all_success() {
        let lines = vec![
            vec![
                Token {
                    token_type: TokenType::Mnemonic,
                    content: "ADD".to_string(),
                    span: Span::new(0, 0..3),
                },
                Token {
                    token_type: TokenType::Register,
                    content: "A".to_string(),
                    span: Span::new(0, 4..5),
                },
                Token {
                    token_type: TokenType::Number,
                    content: "5".to_string(),
                    span: Span::new(0, 6..7),
                },
            ],
            vec![
                Token {
                    token_type: TokenType::Byte,
                    content: ".byte".to_string(),
                    span: Span::new(1, 0..5),
                },
                Token {
                    token_type: TokenType::Number,
                    content: "42".to_string(),
                    span: Span::new(1, 6..8),
                },
            ],
        ];
        let config = Config(vec![InstructionDef {
            mnem: "ADD".to_string(),
            args_def: vec![ArgDef::A, ArgDef::Const],
            mnem_full: "ADDA".to_string(),
            binary: "c0001010".to_string(),
        }]);

        assert!(parse_all(lines, &config).is_ok());
    }

    #[test]
    fn test_parse_all_errors() {
        let lines = vec![
            vec![
                Token {
                    token_type: TokenType::Mnemonic,
                    content: "INVALID".to_string(),
                    span: Span::new(0, 0..7),
                },
                Token {
                    token_type: TokenType::Register,
                    content: "A".to_string(),
                    span: Span::new(0, 8..9),
                },
                Token {
                    token_type: TokenType::Number,
                    content: "5".to_string(),
                    span: Span::new(0, 10..11),
                },
            ],
            vec![
                Token {
                    token_type: TokenType::Byte,
                    content: ".byte".to_string(),
                    span: Span::new(1, 0..5),
                },
                Token {
                    token_type: TokenType::Number,
                    content: "42".to_string(),
                    span: Span::new(1, 6..8),
                },
            ],
        ];
        let config = Config(vec![InstructionDef {
            mnem: "ADD".to_string(),
            args_def: vec![ArgDef::A, ArgDef::Const],
            mnem_full: "ADDA".to_string(),
            binary: "c0001010".to_string(),
        }]);

        let result = parse_all(lines, &config);
        assert!(result.is_err());
        assert!(!result.unwrap_err().is_empty());
    }
}
