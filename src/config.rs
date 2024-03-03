use std::{collections::HashMap, fs::read_to_string, hash::Hash, io, path::Path, str::FromStr};

use serde::Deserialize;
use serde_json::Value;
use thiserror::Error;

use crate::specs::{Mnemonic, Operand};

fn get_config_parse_prefix(line_nr: usize) -> String {
    format!("Failed to parse config on line {}: ", line_nr + 1)
}

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("{0}.")]
    ReadFileError(io::Error),
    #[error("Unknown mnemonic '{0}'.")]
    UnknownMnemonic(String),
    #[error("Unknown operand '{0}'.")]
    UnknownOperand(String),
}

fn print_config_helper(prefix: String, node: &ConfigNode) {
    match node {
        ConfigNode::Leaf(code) => println!("{}{code}", prefix),
        ConfigNode::Branch(branch) => {
            for (key, val) in branch {
                let mut prefix = prefix.clone();
                prefix.push_str(&format!("({:?}) -> ", key));
                print_config_helper(prefix, val);
            }
        }
    }
}

pub fn print_config(config: &Config) {
    for (key, value) in &config.automaton {
        print_config_helper(format!("({:?}) -> ", key), value);
    }
}

#[derive(Debug, PartialEq, Eq, Hash)]
pub enum NodeType {
    Mnemonic(Mnemonic),
    Operand(Operand),
    MachineCode,
}

#[derive(Debug, PartialEq, Eq)]
pub enum ConfigNode {
    Leaf(String),
    Branch(HashMap<NodeType, ConfigNode>),
}

#[derive(Debug)]
pub struct Config {
    pub automaton: HashMap<NodeType, ConfigNode>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
struct InstructionJsonObj {
    mnemonic: String,
    arguments: Vec<String>,
    opcode: String,
    depend_on_flag: String,
}

impl Config {
    // fn parse_instruction(instruction: &str) -> Result<Vec<NodeType>, String> {
    //     let mut iter = instruction.split_whitespace();
    //     let mut nodes: Vec<NodeType> = vec![];

    //     let mnemonic = iter.next().ok_or("Empty instruction.")?;
    //     let mnemonic = Mnemonic::from_str(mnemonic)
    //         .map_err(|_| format!("Unknown mnemonic '{}'.", mnemonic))?;

    //     nodes.push(NodeType::Mnemonic(mnemonic));

    //     for operand in iter {
    //         let operand = Operand::from_str(operand)
    //             .map_err(|_| format!("Unknown operand '{}'.", operand))?;
    //         nodes.push(NodeType::Operand(operand));
    //     }

    //     Ok(nodes)
    // }

    pub fn read_from_file(file_path: impl AsRef<Path>) -> Result<Self, ConfigError> {
        let mut automaton = HashMap::new();

        let content = read_to_string(file_path).map_err(ConfigError::ReadFileError)?;

        let json_objects: HashMap<String, Value> = serde_json::from_str(&content).unwrap();

        let instructions: Vec<InstructionJsonObj> = json_objects
            .iter()
            .map(|(_, v)| serde_json::from_value(v.clone()).unwrap())
            .collect();

        for instruction in &instructions {
            let mnemonic = Mnemonic::new(format!(
                "{}{}",
                instruction.mnemonic, instruction.depend_on_flag
            ));

            let operands = instruction
                .arguments
                .iter()
                .map(|operand| {
                    Operand::from_str(operand)
                        .map_err(|_| ConfigError::UnknownOperand(operand.clone()))
                })
                .collect::<Result<Vec<Operand>, ConfigError>>()?;

            let mut current = &mut automaton;

            for part in vec![NodeType::Mnemonic(mnemonic)]
                .into_iter()
                .chain(operands.into_iter().map(NodeType::Operand))
            {
                current = match current
                    .entry(part)
                    .or_insert_with(|| ConfigNode::Branch(HashMap::new()))
                {
                    ConfigNode::Leaf(_) => unreachable!(),
                    ConfigNode::Branch(next) => next,
                }
            }

            let prev = current.insert(
                NodeType::MachineCode,
                ConfigNode::Leaf(instruction.opcode.clone()),
            );

            assert!(prev.is_none());
        }

        Ok(Self { automaton })
    }
}
