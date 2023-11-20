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
pub enum CheckedLineCode<'a> {
    Byte(Vec<String>),
    Instruction {
        mnemonic_code: &'a str,
        operand_codes: Vec<String>,
    },
}

#[derive(Debug)]
pub struct CheckedLine<'a> {
    pub line: Line<'a>,
    pub code: CheckedLineCode<'a>,
}

fn check_instruction<'a>(
    config: &'a Config,
    labels: &'a HashMap<&'a str, usize>,
    mnemonic: &Token,
    operands: &Vec<(Operand, &Token)>,
) -> Result<CheckedLineCode<'a>, WriterErr> {
    let mnemonic = match &mnemonic.token_type {
        TokenType::Mnemonic(mnemonic) => mnemonic,
        _ => return Err(WriterErr::UnknownMnemonic(mnemonic.content.clone())),
    };

    let mut current_node = config
        .automaton
        .get(&NodeType::Mnemonic(*mnemonic))
        .unwrap();

    let mut operand_binary_codes = vec![];

    for operand in operands {
        let parsed_operand = parse_value(labels, operand.1);
        if let Some(operand_code) = parsed_operand {
            let operand_code = operand_code?;
            operand_binary_codes.push(operand_code)
        };
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
            mnemonic_code,
            operand_codes: operand_binary_codes,
        }),
        _ => unreachable!(),
    }
}

fn parse_num(number: i64) -> Result<String, WriterErr> {
    if !(-128..=255).contains(&number) {
        return Err(WriterErr::NumberOutOfRange(number));
    }

    // TODO: Add a unified format for parse_num and parse_labelref (remove code dup)
    let binary = format!("{:08b}", number);
    Ok(binary)
}

fn parse_labelref<'a>(
    labels: &'a HashMap<&'a str, usize>,
    label: &str,
) -> Result<String, WriterErr> {
    let label = labels
        .get(label)
        .ok_or(WriterErr::UnknownLabel(label.to_string()))?;
    let binary = format!("{:08b}", label);
    Ok(binary)
}

fn parse_value<'a>(
    labels: &'a HashMap<&'a str, usize>,
    value: &Token,
) -> Option<Result<String, WriterErr>> {
    match &value.token_type {
        TokenType::Number(number) => Some(parse_num(*number)),
        TokenType::LabelRef(label_ref) => Some(parse_labelref(labels, label_ref)),
        _ => None,
    }
}

fn check_byte<'a>(
    labels: &'a HashMap<&'a str, usize>,
    declared_values: &Vec<&Token>,
) -> Result<CheckedLineCode<'a>, WriterErr> {
    let mut parsed_values = vec![];
    for value in declared_values {
        let parsed_value = parse_value(labels, value);
        if let Some(parsed_value_result) = parsed_value {
            parsed_values.push(parsed_value_result?)
        };
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
