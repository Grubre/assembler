use assembler::*;
use std::collections::HashMap;
use std::{error::Error, fs};

enum Arg {
    Register(String),
    ImmediateValue(i64),
}

enum Instruction {
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

fn parse(tokens: &[Token]) -> Vec<String> {
    let lines = Vec::new();
    let labels = populate_labels(&tokens);

    while let Some(token) = tokens.iter().next() {
        match &token.token_type {
            TokenType::Mnemonic => {}
            TokenType::Number => {}
            TokenType::Register => {}
            TokenType::Label => {}
            TokenType::LabelRef => {
                let label_ref = token.content.trim_start_matches('#');
                if let Some(label) = labels.get(label_ref) {
                    //...
                } else {
                    println!("ERROR: Unknown label at line {}", token.line_nr);
                    std::process::exit(1);
                }
            }
            TokenType::Comment => {}
        }
    }
    lines
}

fn main() {
    let contents = fs::read_to_string("./test.as").expect("Failed to read the file");

    for token in tokenize(&create_patterns(), &contents) {
        println!("{:?}, {}", token.token_type, token.content);
    }
}
