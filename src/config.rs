use std::{collections::HashMap, fs::read_to_string, hash::Hash, io, path::Path, str::FromStr};

use thiserror::Error;

use crate::specs::{Mnemonic, Operand};

fn get_config_parse_prefix(line_nr: usize) -> String {
    format!("Failed to parse config on line {}: ", line_nr + 1)
}

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("{0}.")]
    ReadFileError(io::Error),
    #[error("{}Each the row of config should have three comma separated values.", get_config_parse_prefix(*.0))]
    NotEnoughColumns(usize),
    #[error("{}{}", get_config_parse_prefix(*.0), .1)]
    ParseInstructionError(usize, String),
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

impl Config {
    fn parse_instruction(instruction: &str) -> Result<Vec<NodeType>, String> {
        let mut iter = instruction.split_whitespace();
        let mut nodes: Vec<NodeType> = vec![];

        let mnemonic = iter.next().ok_or("Empty instruction.")?;
        let mnemonic = Mnemonic::from_str(mnemonic)
            .map_err(|_| format!("Unknown mnemonic '{}'.", mnemonic))?;

        nodes.push(NodeType::Mnemonic(mnemonic));

        for operand in iter {
            let operand = Operand::from_str(operand)
                .map_err(|_| format!("Unknown operand '{}'.", operand))?;
            nodes.push(NodeType::Operand(operand));
        }

        Ok(nodes)
    }

    pub fn read_from_file(file_path: impl AsRef<Path>) -> Result<Self, ConfigError> {
        let mut automaton = HashMap::new();

        let content = read_to_string(file_path).map_err(ConfigError::ReadFileError)?;
        let lines = content
            .lines()
            .map(|line| line.split(',').map(|str| str.trim()).collect::<Vec<&str>>());

        for (i, line) in lines.enumerate() {
            let instruction = *line.first().ok_or(ConfigError::NotEnoughColumns(i))?;
            let alternative = *line.get(1).ok_or(ConfigError::NotEnoughColumns(i))?;
            let machine_code = *line.get(2).ok_or(ConfigError::NotEnoughColumns(i))?;

            let instruction = Config::parse_instruction(instruction)
                .map_err(|message| ConfigError::ParseInstructionError(i, message))?;

            let mut current = &mut automaton;
            for part in instruction {
                current = match current
                    .entry(part)
                    .or_insert_with(|| ConfigNode::Branch(HashMap::new()))
                {
                    ConfigNode::Leaf(_) => unreachable!(),
                    ConfigNode::Branch(next) => next,
                }
            }

            current.insert(
                NodeType::MachineCode,
                ConfigNode::Leaf(String::from(machine_code)),
            );
        }

        Ok(Self { automaton })
    }
}
