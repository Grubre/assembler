use regex::Regex;
use thiserror::Error;

use crate::error::*;

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub enum TokenType {
    Mnemonic,
    Register,
    Number,
    MemAddress,
    Label,
    LabelRef,
    LabelAddressRef,
    Comment,
    Byte,
}

#[derive(Debug)]
pub struct Token {
    pub token_type: TokenType,
    pub content: String,
    pub span: Span,
}

#[derive(Debug, Error)]
pub enum LexerErr {
    #[error("Unknown token")]
    UnknownToken,
}

pub fn create_patterns() -> Vec<(TokenType, Regex)> {
    let patterns = vec![
        (
            TokenType::Mnemonic,
            Regex::new(r"(?i)^(NOP|MOV|PUSH|POP|JMP|ADD|SUB|OR|AND|NEG|INV|SHR|SHL|CMP|HALT)(\s|\n|$)")
                .unwrap(),
        ),
        (TokenType::Register, Regex::new(r"^(A|B|F)(\s|\n|$)").unwrap()),
        (
            TokenType::Number,
            Regex::new(r"^(-?0x[0-9A-Fa-f]+|0b[01]+|-?0o[0-7]+|-?[0-9]+)").unwrap(),
        ),
        (
            TokenType::MemAddress,
            Regex::new(r"^\[(0x[0-9A-Fa-f]+|0b[01]+|0o[0-7]+|[0-9]+)\](\s|\n|$)").unwrap(),
        ),
        (
            TokenType::Label,
            Regex::new(r"^([a-zA-Z_][a-zA-Z0-9_]*):").unwrap(),
        ),
        (
            TokenType::LabelRef,
            Regex::new(r"^#([a-zA-Z_][a-zA-Z0-9_]*)(\s|\n|$)").unwrap(),
        ),
        (
            TokenType::LabelAddressRef,
            Regex::new(r"^\[#([a-zA-Z_][a-zA-Z0-9_]*)\](\s|\n|$)").unwrap(),
        ),
        (TokenType::Comment, Regex::new(r"^;(.*)$").unwrap()),
        (TokenType::Byte, Regex::new(r"^(byte)(\s|\n|$)").unwrap()),
    ];

    patterns
}

pub fn tokenize<'a>(
    patterns: &'a Vec<(TokenType, Regex)>,
    input: &'a str,
) -> Result<Vec<Vec<Token>>, Vec<Error>> {
    let mut tokens = Vec::new();
    let mut errors = Vec::new();

    for (i, line) in input.lines().enumerate() {
        let line_ref = line.trim_start();
        let mut char_index_accumulator = line.len() - line_ref.len();
        let mut line_ref = line_ref.trim_end();
        let mut line_tokens = Vec::new();

        while !line_ref.is_empty() {
            let mut matched = false;

            for (token_type, pattern) in patterns {
                if let Some(word) = pattern.captures(line_ref) {
                    let content = word.get(1).unwrap().as_str();
                    let word_len = word.get(0).unwrap().end();
                    line_tokens.push(Token {
                        token_type: token_type.clone(),
                        content: content.to_string(),
                        span: Span::new(
                            i,
                            char_index_accumulator..(char_index_accumulator + word_len),
                        ),
                    });
                    let next_line_ref = &line_ref[word_len..];
                    let next_line_ref_trimmed = next_line_ref.trim_start();
                    let whitespace_len = next_line_ref.len() - next_line_ref_trimmed.len();
                    char_index_accumulator += word_len + whitespace_len;
                    line_ref = next_line_ref_trimmed;
                    matched = true;
                    break;
                }
            }

            if !matched {
                let split = line_ref.split_once(' ').unwrap_or((line_ref, ""));
                let tok = split.0;
                line_ref = split.1;
                errors.push(LexerErr::UnknownToken.with_span(Span::new(
                    i,
                    char_index_accumulator..(char_index_accumulator + tok.len()),
                )));
                char_index_accumulator += tok.len() + 1;
            }
        }

        let line_tokens: Vec<_> = line_tokens
            .into_iter()
            .filter(|tok| tok.token_type != TokenType::Comment)
            .collect();

        if !line_tokens.is_empty() {
            tokens.push(line_tokens)
        }
    }

    if errors.is_empty() {
        Ok(tokens)
    } else {
        Err(errors)
    }
}
