use std::collections::HashMap;

use thiserror::Error;

use crate::{
    config::{Config, ConfigNode, NodeType},
    parser::Line,
    specs::Operand,
    token::{Token, TokenType},
};

#[derive(PartialEq, Eq, Debug, Error)]
pub enum WriterErr {
    #[error("Unknown mnemonic '{0}'.")]
    UnknownMnemonic(String),
    #[error("Invalid operand '{0}'.")]
    InvalidOperand(String),
    #[error("Number should be in range [-128, 255], instead found {0}.")]
    NumberOutOfRange(i64),
    #[error("Unknown label '{0}'.")]
    UnknownLabel(String),
}

#[derive(Debug)]
pub enum CheckedLineCode {
    Byte(Vec<u8>),
    Instruction {
        mnemonic_code: u8,
        operand_codes: Vec<u8>,
    },
}

#[derive(Debug)]
pub struct CheckedLine<'a> {
    pub line: Line<'a>,
    pub code: CheckedLineCode,
}

fn check_instruction<'a>(
    config: &'a Config,
    labels: &'a HashMap<&'a str, usize>,
    mnemonic: &Token,
    operands: &Vec<(Operand, &Token)>,
) -> Result<CheckedLineCode, WriterErr> {
    let mnemonic = match &mnemonic.token_type {
        TokenType::Mnemonic(mnemonic) => mnemonic,
        _ => return Err(WriterErr::UnknownMnemonic(mnemonic.content.clone())),
    };

    let mut current_node = config
        .automaton
        .get(&NodeType::Mnemonic(mnemonic.clone()))
        .unwrap();

    let mut operand_binary_codes = vec![];

    for operand in operands {
        match operand.0 {
            Operand::Mem8 | Operand::Const => {
                let parsed_operand = parse_value(labels, operand.1)?;
                operand_binary_codes.push(parsed_operand);
            },
            Operand::Mem16 => {
                let parsed_operand = parse_wide_value(labels, operand.1)?;
                let [higher, lower] = parsed_operand.to_be_bytes();
                operand_binary_codes.push(higher);
                operand_binary_codes.push(lower);
            },
            _ => {}
        }
        match current_node {
            crate::config::ConfigNode::Branch(children) => {
                match children.get(&NodeType::Operand(operand.0)) {
                    Some(next) => current_node = next,
                    None => return Err(WriterErr::InvalidOperand(operand.1.content.clone())),
                }
            }
            _ => {
                unreachable!();
            }
        };
    }

    let ConfigNode::Branch(leaf) = current_node else {
        unreachable!();
    };

    let Some(leaf) = leaf.get(&NodeType::MachineCode) else {
        unreachable!();
    };

    match leaf {
        ConfigNode::Leaf(mnemonic_code) => Ok(CheckedLineCode::Instruction {
            mnemonic_code: binary_str_to_byte(mnemonic_code),
            operand_codes: operand_binary_codes,
        }),
        _ => unreachable!(),
    }
}

// TODO: Check whether keeping the mnemonic_code as String is better than keeping it as u8
//       (in terms of performance).
fn binary_str_to_byte(binary_str: &str) -> u8 {
    let mut byte = 0;
    for (i, c) in binary_str.chars().rev().enumerate() {
        if c == '1' {
            byte |= 1 << i;
        }
    }
    byte
}

fn parse_num(number: i64) -> Result<u8, WriterErr> {
    if !(-128..=255).contains(&number) {
        return Err(WriterErr::NumberOutOfRange(number));
    }

    Ok(number as u8)
}

fn parse_labelref<'a>(
    labels: &'a HashMap<&'a str, usize>,
    label: &str,
) -> Result<u8, WriterErr> {
    let label = labels
        .get(label)
        .ok_or(WriterErr::UnknownLabel(label.to_string()))?;
    Ok(*label as u8)
}

fn parse_value<'a>(
    labels: &'a HashMap<&'a str, usize>,
    value: &Token,
) -> Result<u8, WriterErr> {
    match &value.token_type {
        TokenType::Number(number) => parse_num(*number),
        TokenType::LabelRef(label_ref) => parse_labelref(labels, label_ref),
        _ => unreachable!(),
    }
}

fn parse_wide_num(number: i64) -> Result<u16, WriterErr> {
    if !(-32_768..=65_535).contains(&number) {
        return Err(WriterErr::NumberOutOfRange(number));
    }

    Ok(number as u16)
}

fn parse_wide_labelref<'a>(
    labels: &'a HashMap<&'a str, usize>,
    label: &str,
) -> Result<u16, WriterErr> {
    let label = labels
        .get(label)
        .ok_or(WriterErr::UnknownLabel(label.to_string()))?;
    Ok(*label as u16)
}

fn parse_wide_value<'a>(
    labels: &'a HashMap<&'a str, usize>,
    value: &Token,
) -> Result<u16, WriterErr> {
    match &value.token_type {
        TokenType::Number(number) => parse_wide_num(*number),
        TokenType::LabelRef(label_ref) => parse_wide_labelref(labels, label_ref),
        _ => unreachable!(),
    }
}

fn check_byte<'a>(
    labels: &'a HashMap<&'a str, usize>,
    declared_values: &Vec<&Token>,
) -> Result<CheckedLineCode, WriterErr> {
    let mut parsed_values = vec![];
    for value in declared_values {
        let parsed_value = parse_value(labels, value);
        parsed_values.push(parsed_value?);
    }
    Ok(CheckedLineCode::Byte(parsed_values))
}

pub fn check_semantics<'a>(
    lines: Vec<Line<'a>>,
    labels: &'a HashMap<&'a str, usize>,
    config: &'a Config,
) -> Result<Vec<CheckedLine<'a>>, WriterErr> {
    let mut checked_lines: Vec<_> = vec![];

    for line in lines {
        let code = match &line {
            Line::Byte(declared_values) => check_byte(labels, declared_values),
            Line::Instruction { mnemonic, operands } => {
                check_instruction(config, labels, mnemonic, operands)
            }
        }?;
        checked_lines.push(CheckedLine { line, code });
    }

    Ok(checked_lines)
}
