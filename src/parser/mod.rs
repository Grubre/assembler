use super::lexer::{Token, TokenType};
use std::collections::HashMap;

pub enum Arg {
    Register(String),
    ImmediateValue(i64),
}

pub enum Instruction {
    Nop,
    Mov(Arg, Arg),
    Push(Arg),
    Pop(Arg),
    Jmp(Arg),
    Add(Arg),
    Sub(Arg, Arg, Arg),
    Or(Arg),
    And(Arg),
    Neg(Arg, Arg),
    Inv(Arg),
    Shr(Arg, Arg),
    Shl(Arg, Arg),
    Cmp(Arg, Arg),
    Halt,
}

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


fn populate_labels(tokens: &[Token]) -> HashMap<&str, usize> {
    let mut labels = HashMap::new();
    while let Some(token) = tokens.iter().next() {
        match &token.token_type {
            TokenType::Label => {
                let label = token.content.trim_end_matches(':');
                labels.insert(label, token.line_nr);
            }
            _ => {}
        }
    }
    labels
}

fn parse_instruction<'a>(token: &Token, iter: &mut impl Iterator<Item = &'a Token>) {

}

pub fn parse(tokens: &[Token]) -> Vec<String> {
    let lines = Vec::new();
    let labels = populate_labels(&tokens);
    let mut iter = tokens.iter();

    while let Some(token) = iter.next() {
        match &token.token_type {
            TokenType::Mnemonic => {
                let args_cnt = parse_instruction(&token, &mut iter);

            }
            TokenType::Label => {}
            // TokenType::LabelRef => {
            //     let label_ref = token.content.trim_start_matches('#');
            //     if let Some(label) = labels.get(label_ref) {
            //         //...
            //     } else {
            //         println!("ERROR: Unknown label at line {}", token.line_nr);
            //         std::process::exit(1);
            //     }
            // }
            TokenType::Comment => {}
            _ => {
                println!("ERROR: parsing error at line {}", token.line_nr);
                std::process::exit(1);
            }
        }
    }
    lines
}


