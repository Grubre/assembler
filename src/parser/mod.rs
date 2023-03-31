use super::lexer::{Token, TokenType};
use std::collections::HashMap;

pub fn parse_number(str: &str) -> Result<i64, std::num::ParseIntError> {
    if str.starts_with("0x") {
        return i64::from_str_radix(&str[2..], 16);
    }
    if str.starts_with("0b") {
        return i64::from_str_radix(&str[2..], 2);
    }
    if str.starts_with("0") {
        return i64::from_str_radix(str, 8);
    }

    return i64::from_str_radix(str, 10);
}

pub fn parse_mem_address(str: &str) -> Result<i64, std::num::ParseIntError> {
    todo!()
}

pub enum Arg<'a> {
    Register(&'a str),
    ImmediateValue(i64),
    MemAddress(i64),
    LabelRef(String),
}

#[derive(Debug, PartialEq, Eq)]
pub struct TokenToArgError;

impl<'a> TryFrom<&'a Token> for Arg<'a> {
    type Error = TokenToArgError;

    fn try_from(token: &'a Token) -> Result<Self, Self::Error> {
        match &token.token_type {
            TokenType::Register => Ok(Arg::Register(&token.content)),
            TokenType::Number => Ok(Arg::ImmediateValue(parse_number(&token.content).unwrap())),
            TokenType::MemAddress => {
                todo!();
                Ok(Arg::MemAddress(parse_mem_address(&token.content).unwrap()))
            }
            _ => Err(TokenToArgError {}),
        }
    }
}

pub enum Instruction<'a> {
    Nop,
    Mov(Arg<'a>, Arg<'a>),
    Push(Arg<'a>),
    Pop(Arg<'a>),
    Jmp(Arg<'a>),
    Add(Arg<'a>),
    Sub(Arg<'a>, Arg<'a>, Arg<'a>),
    Or(Arg<'a>),
    And(Arg<'a>),
    Neg(Arg<'a>, Arg<'a>),
    Inv(Arg<'a>, Arg<'a>),
    Shr(Arg<'a>, Arg<'a>),
    Shl(Arg<'a>, Arg<'a>),
    Cmp(Arg<'a>, Arg<'a>),
    Halt,
}

fn populate_labels<'a>(lines: &[&'a [Token]]) -> HashMap<&'a str, usize> {
    let mut labels = HashMap::new();
    for line in lines {
        if let Some(token) = line.first() {
            if token.token_type == TokenType::Label {
                let label = token.content.trim_end_matches(':');
                labels.insert(label, token.line_nr);
            }
        }
    }
    labels
}

fn parse_instruction(instruction: Instruction) -> String {
    use Arg::*;
    use Instruction::*;
    match instruction {
        Nop => todo!(),
        Mov(Register(dest), Register(src)) => todo!(),
        Mov(Register(dest), ImmediateValue(val)) => todo!(),
        Mov(Register(dest), MemAddress(mem)) => todo!(),
        Mov(MemAddress(mem), Register(src)) => todo!(),
        Push(Register(reg)) => todo!(),
        Push(ImmediateValue(val)) => todo!(),
        Push(MemAddress(mem)) => todo!(),
        Pop(Register(reg)) => todo!(),
        Pop(MemAddress(reg)) => todo!(),
        Jmp(_) => todo!(),
        Add(MemAddress(reg)) => todo!(),
        Sub(Register(op1), Register(op2), Register(dest)) => todo!(),
        Sub(Register(op1), Register(op2), MemAddress(dest)) => todo!(),
        Or(Register(dest)) => todo!(),
        Or(MemAddress(dest)) => todo!(),
        And(Register(dest)) => todo!(),
        And(MemAddress(dest)) => todo!(),
        Neg(Register(op), Register(dest)) => todo!(),
        Neg(Register(op), MemAddress(dest)) => todo!(),
        Inv(Register(op), Register(dest)) => todo!(),
        Inv(Register(op), MemAddress(dest)) => todo!(),
        Shr(Register(op), Register(dest)) => todo!(),
        Shr(Register(op), MemAddress(dest)) => todo!(),
        Shl(Register(op), Register(dest)) => todo!(),
        Shl(Register(op), MemAddress(dest)) => todo!(),
        Cmp(Register(reg1), Register(reg2)) => todo!(),
        Halt => todo!(),
        _ => panic!(),
    }
}

fn parse_line<'a>(labels: &HashMap<&str, usize>, tokens: &[Token]) -> String {
    use Instruction::*;

    let mut iter = tokens.iter();

    let mut mnemonic = iter.next().unwrap();
    if tokens.first().unwrap().token_type == TokenType::Label {
        mnemonic = iter.next().unwrap();
    }

    let arg1 = Arg::try_from(iter.next().unwrap());
    let arg2 = Arg::try_from(iter.next().unwrap());
    let arg3 = Arg::try_from(iter.next().unwrap());

    match mnemonic.content.as_str() {
        "nop" => parse_instruction(Nop),
        "mov" => parse_instruction(Instruction::Mov(arg2.unwrap(), arg1.unwrap())),
        "push" => parse_instruction(Instruction::Push(arg1.unwrap())),
        "pop" => parse_instruction(Instruction::Pop(arg1.unwrap())),
        "jmp" => parse_instruction(Instruction::Jmp(arg1.unwrap())),
        "add" => parse_instruction(Instruction::Add(arg1.unwrap())),
        "sub" => parse_instruction(Instruction::Sub(
            arg1.unwrap(),
            arg2.unwrap(),
            arg3.unwrap(),
        )),
        "or" => parse_instruction(Instruction::Or(arg1.unwrap())),
        "and" => parse_instruction(Instruction::And(arg1.unwrap())),
        "neg" => parse_instruction(Instruction::Neg(arg1.unwrap(), arg2.unwrap())),
        "inv" => parse_instruction(Instruction::Inv(arg1.unwrap(), arg2.unwrap())),
        "shr" => parse_instruction(Instruction::Shr(arg1.unwrap(), arg2.unwrap())),
        "shl" => parse_instruction(Instruction::Shl(arg1.unwrap(), arg2.unwrap())),
        "cmp" => parse_instruction(Instruction::Cmp(arg1.unwrap(), arg2.unwrap())),
        "halt" => parse_instruction(Instruction::Halt {}),
        _ => {
            unreachable!()
        }
    }
}

pub fn parse(lines: &[&[Token]]) -> Vec<String> {
    let mut output = Vec::new();
    let labels = populate_labels(&lines);

    for line in lines {
        let str = parse_line(&labels, line);
        output.push(str);
    }
    output
}
