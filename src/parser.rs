use crate::config::{ArgDef, Config, InstructionDef};

use super::lexer::{Token, TokenType};
use std::collections::HashMap;

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
pub enum Arg {
    Register(Register),
    ImmediateValue(Value),
    MemAddress(Value),
}

#[derive(Debug)]
pub struct Instruction {
    mnem: String,
    args: Vec<Arg>,
}

#[derive(Debug)]
pub struct Labels(pub HashMap<String, usize>);

///MATCHING

impl Arg {
    pub fn matches_def(&self, def: &ArgDef) -> bool {
        match self {
            Arg::Register(Register::A) => *def == ArgDef::A,
            Arg::Register(Register::B) => *def == ArgDef::B,
            Arg::Register(Register::F) => *def == ArgDef::F,
            Arg::ImmediateValue(_) => *def == ArgDef::Const,
            Arg::MemAddress(_) => *def == ArgDef::Mem,
        }
    }
}

impl Instruction {
    fn matches_def(&self, def: &InstructionDef) -> bool {
        self.mnem == def.mnem
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

#[derive(Debug, PartialEq, Eq)]
pub struct TokenToArgError;

impl TryFrom<&Token> for Arg {
    type Error = TokenToArgError;

    fn try_from(token: &Token) -> Result<Self, Self::Error> {
        match &token.token_type {
            TokenType::Register => match token.content.as_str() {
                "A" => Ok(Arg::Register(Register::A)),
                "B" => Ok(Arg::Register(Register::B)),
                _ => unreachable!(),
            },
            TokenType::Number => Ok(Arg::ImmediateValue(Value::Num(
                parse_number(&token.content).unwrap(),
            ))),
            TokenType::LabelRef => Ok(Arg::ImmediateValue(Value::LabelRef(token.content.clone()))),
            TokenType::MemAddress => Ok(Arg::MemAddress(Value::Num(
                parse_number(&token.content).unwrap(),
            ))),
            TokenType::LabelAddressRef => {
                Ok(Arg::MemAddress(Value::LabelRef(token.content.clone())))
            }
            _ => Err(TokenToArgError {}),
        }
    }
}

#[derive(Debug)]
pub enum Unresolved {
    LabelRef(String),
    Value(String),
}

impl Arg {
    pub fn to_unresolved_binary(&self) -> Option<Unresolved> {
        match self {
            Arg::Register(_) => None,
            Arg::ImmediateValue(Value::Num(num)) => Some(Unresolved::Value(format!("{:08b}", num))),
            Arg::MemAddress(Value::Num(num)) => Some(Unresolved::Value(format!("{:08b}", num))),
            Arg::ImmediateValue(Value::LabelRef(label)) => {
                Some(Unresolved::LabelRef(label.clone()))
            }
            Arg::MemAddress(Value::LabelRef(label)) => Some(Unresolved::LabelRef(label.clone())),
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
    tokens: &[Token],
    config: &Config,
) -> Vec<Unresolved> {
    let mut iter = tokens.iter().skip_while(|token| {
        if token.token_type == TokenType::Label {
            labels.insert(token.content.to_owned(), *curr_line);
            true
        } else {
            false
        }
    });

    let mnem = iter.next().unwrap();
    assert_eq!(mnem.token_type, TokenType::Mnemonic);
    let mnem = mnem.content.clone();

    let args: Vec<_> = iter.map(|token| token.try_into().unwrap()).collect();

    let instr = Instruction { mnem, args };

    println!("{instr:?}");

    let def = instr.find_match(config).unwrap();

    let mut ret = Vec::new();

    ret.push(Unresolved::Value(def.binary.clone()));

    for arg in &instr.args {
        if let Some(unres) = arg.to_unresolved_binary() {
            ret.push(unres);
        }
    }

    *curr_line += ret.len();

    ret
}

pub fn parse_all(lines: &[impl AsRef<[Token]>], config: &Config) -> (Vec<Unresolved>, Labels) {
    let mut unresolved = Vec::new();
    let mut labels = Labels(HashMap::new());

    let mut curr_line = 0;
    for line in lines {
        let mut ret = parse_line(&mut labels.0, &mut curr_line, line.as_ref(), config);
        unresolved.append(&mut ret);
    }

    (unresolved, labels)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test() {
        let instr = Instruction {
            mnem: "SUB".to_string(),
            args: vec![
                Arg::MemAddress(Value::Num(5)),
                Arg::Register(Register::A),
                Arg::Register(Register::B),
            ],
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
        assert!(Arg::MemAddress(Value::Num(5)).matches_def(&ArgDef::Mem));
    }
}
