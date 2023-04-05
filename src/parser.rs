use thiserror::Error;

use crate::{
    config::{ArgDef, Config, InstructionDef},
    error::{Error, Span, WithSpan},
};

use super::lexer::{Token, TokenType};
use std::collections::HashMap;

#[derive(Debug, Error)]
pub enum ParseErr {
    #[error("Can't find mapping")]
    NoMapping,
    #[error("Isn't an argument for instruction")]
    NotAnArg,
}

#[derive(Debug)]
pub enum Register {
    A,
    B,
    F,
}

#[derive(Debug)]
pub enum Value {
    Num(i64),
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

impl TryFrom<Token> for Arg {
    type Error = Error;

    fn try_from(token: Token) -> Result<Self, Self::Error> {
        match &token.token_type {
            TokenType::Register => match token.content.as_str() {
                "A" => Ok(ArgKind::Register(Register::A)),
                "B" => Ok(ArgKind::Register(Register::B)),
                _ => unreachable!(),
            },
            TokenType::Number => Ok(ArgKind::ImmediateValue(Value::Num(
                parse_number(&token.content).unwrap(),
            ))),
            TokenType::LabelRef => Ok(ArgKind::ImmediateValue(Value::LabelRef(
                token.content.clone(),
            ))),
            TokenType::MemAddress => Ok(ArgKind::MemAddress(Value::Num(
                parse_number(&token.content).unwrap(),
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
                Some(Unresolved::LabelRef(label.clone(), self.span))
            }
            ArgKind::MemAddress(Value::LabelRef(label)) => {
                Some(Unresolved::LabelRef(label.clone(), self.span))
            }
        }
    }
}

pub fn parse_number(str: &str) -> Result<i64, std::num::ParseIntError> {
    if let Some(num) = str.strip_prefix("0x") {
        i64::from_str_radix(num, 16)
    } else if let Some(num) = str.strip_prefix("0b") {
        i64::from_str_radix(num, 2)
    } else if let Some(num) = str.strip_prefix("0o") {
        i64::from_str_radix(num, 8)
    } else {
        str.parse::<i64>()
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

    let first_token = iter.next().unwrap();
    let mut ret = Vec::new();

    match first_token.token_type {
        TokenType::Mnemonic => {
            let content = first_token.content;

            let (args_ok, args_err): (Vec<Result<_, _>>, Vec<Result<_, _>>) =
                iter.map(|token| token.try_into()).partition(Result::is_ok);

            if !args_err.is_empty() {
                let err = args_err.into_iter().map(|arg| arg.unwrap_err()).collect();
                return Err(err);
            }

            let args = args_ok.into_iter().map(|arg| arg.unwrap()).collect();

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
            let value = iter.next().unwrap();
            ret.push(Unresolved::Value(format!(
                "{:08b}",
                parse_number(&value.content).unwrap()
            )));
        }
        _ => {
            panic!();
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
            span: dummy_span.clone(),
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
}
