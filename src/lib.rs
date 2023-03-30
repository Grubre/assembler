use regex::Regex;
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

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub enum TokenType {
    Mnemonic,
    Register,
    Number,
    Label,
    LabelRef,
    Comment,
}

#[derive(Debug, PartialEq, Hash)]
pub struct Token {
    pub token_type: TokenType,
    pub content: String,
    pub line_nr: usize,
}

pub fn create_patterns() -> HashMap<TokenType, Regex> {
    let mut patterns = HashMap::new();

    patterns.insert(
        TokenType::Mnemonic,
        Regex::new(r"(?i)^(NOP|MOV|PUSH|POP|JMP|ADD|SUB|OR|AND|NEG|INV|SHR|SHL|CMP|HALT)").unwrap(),
    );
    patterns.insert(TokenType::Register, Regex::new(r"^(A|B)").unwrap());
    patterns.insert(
        TokenType::Number,
        Regex::new(r"^(0x[0-9A-Fa-f]+|0b[01]+|0o[0-7]+|[0-9]+)$").unwrap(),
    );
    patterns.insert(
        TokenType::Label,
        Regex::new(r"^([a-zA-Z_][a-zA-Z0-9_]*):").unwrap(),
    );
    patterns.insert(
        TokenType::LabelRef,
        Regex::new(r"^#([a-zA-Z_][a-zA-Z0-9_]*)").unwrap(),
    );
    patterns.insert(TokenType::Comment, Regex::new(r"^;.*").unwrap());

    patterns
}

pub fn tokenize(patterns: &HashMap<TokenType, Regex>, input: &str) -> Vec<Token> {
    let mut tokens = Vec::new();

    for (i, line) in input.lines().enumerate() {
        let mut line_ref = line.trim();

        while !line_ref.is_empty() {
            let mut matched = false;

            for (token_type, pattern) in patterns {
                if let Some(word) = pattern.find(line_ref) {
                    let content = word.as_str();
                    tokens.push(Token {
                        token_type: token_type.clone(),
                        content: content.to_string(),
                        line_nr: i,
                    });
                    line_ref = line_ref[word.end()..].trim_start();
                    matched = true;
                    break;
                }
            }

            if !matched {
                println!("ERROR: Unknown token on line {}", i + 1);
                std::process::exit(1);
            }
        }
    }
    tokens
}
