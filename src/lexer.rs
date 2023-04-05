use std::ops::Range;

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
impl PartialEq for Token {
    fn eq(&self, other: &Self) -> bool {
        self.token_type == other.token_type && self.content == other.content
    }
}
impl Eq for Token {}
impl Token {
    pub fn new(token_type: TokenType, content: &str, line: usize, range: Range<usize>) -> Self {
        Token {
            token_type,
            content: content.to_string(),
            span: Span::new(line, range),
        }
    }
}
#[derive(PartialEq, Eq, Debug, Error)]
pub enum LexerErr {
    #[error("Unknown token")]
    UnknownToken,
}

pub fn create_patterns() -> Vec<(TokenType, Regex)> {
    let patterns = vec![
        (
            TokenType::Mnemonic,
            Regex::new(
                r"(?i)^(NOP|MOV|PUSH|POP|JMP|ADD|SUB|OR|AND|NEG|INV|SHR|SHL|CMP|HALT)(\s|\n|$)",
            )
            .unwrap(),
        ),
        (
            TokenType::Register,
            Regex::new(r"^(A|B|F)(\s|\n|$)").unwrap(),
        ),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_patterns() {
        let patterns = create_patterns();

        // Test TokenType::Mnemonic
        let mnemonic_re = &patterns[0].1;
        assert!(mnemonic_re.is_match("MOV "));
        assert!(mnemonic_re.is_match("push\n"));
        assert!(mnemonic_re.is_match("Jmp"));
        assert!(!mnemonic_re.is_match("INVALID"));

        // Test TokenType::Register
        let register_re = &patterns[1].1;
        assert!(register_re.is_match("A "));
        assert!(register_re.is_match("B\n"));
        assert!(!register_re.is_match("C "));

        // Test TokenType::Number
        let number_re = &patterns[2].1;
        assert!(number_re.is_match("42"));
        assert!(number_re.is_match("-0o52"));

        // Test TokenType::MemAddress
        let mem_address_re = &patterns[3].1;
        assert!(mem_address_re.is_match("[42] "));
        assert!(mem_address_re.is_match("[0x2A]\n"));
        assert!(!mem_address_re.is_match("42"));

        // Test TokenType::Label
        let label_re = &patterns[4].1;
        assert!(label_re.is_match("start:"));
        assert!(label_re.is_match("test:"));
        assert!(label_re.is_match("a:"));

        // Test TokenType::LabelRef
        let label_ref_re = &patterns[5].1;
        assert!(label_ref_re.is_match("#start "));
        assert!(label_ref_re.is_match("#label1\n"));
        assert!(label_ref_re.is_match("#test"));

        // Test TokenType::LabelAddressRef
        let label_address_ref_re = &patterns[6].1;
        assert!(label_address_ref_re.is_match("[#start] "));
        assert!(label_address_ref_re.is_match("[#label1]\n"));
        assert!(label_address_ref_re.is_match("[#test]"));

        // Test TokenType::Comment
        let comment_re = &patterns[7].1;
        assert!(comment_re.is_match("; This is a comment"));
        assert!(comment_re.is_match(";Another comment"));

        // Test TokenType::Byte
        let byte_re = &patterns[8].1;
        assert!(byte_re.is_match("byte "));
        assert!(byte_re.is_match("byte\n"));
        assert!(!byte_re.is_match("bytes "));
    }

    #[test]
    fn test_tokenize() {
        let patterns = create_patterns();
        let input = "
            start: MOV A 42
            ADD A
            byte 0x05 -0x05 0b001 10 0o12 -10 -0o12
            JMP #start
            a: SUB [0x2A] A B
            b: SUB [#label2] B A
            HALT
        ";

        let result = tokenize(&patterns, input);

        assert!(result.is_ok());
        let tokens = result.unwrap();

        assert_eq!(
            tokens[0][0],
            Token::new(TokenType::Label, "start", 1, 12..17)
        );
        assert_eq!(
            tokens[0][1],
            Token::new(TokenType::Mnemonic, "MOV", 1, 18..21)
        );
        assert_eq!(
            tokens[0][2],
            Token::new(TokenType::Register, "A", 1, 22..23)
        );
        assert_eq!(tokens[0][3], Token::new(TokenType::Number, "42", 1, 24..26));

        assert_eq!(
            tokens[1][0],
            Token::new(TokenType::Mnemonic, "ADD", 2, 8..11)
        );
        assert_eq!(
            tokens[1][1],
            Token::new(TokenType::Register, "A", 2, 12..13)
        );

        assert_eq!(tokens[2][0], Token::new(TokenType::Byte, "byte", 3, 8..12));
        assert_eq!(
            tokens[2][1],
            Token::new(TokenType::Number, "0x05", 3, 13..17)
        );
        assert_eq!(
            tokens[2][2],
            Token::new(TokenType::Number, "-0x05", 3, 18..23)
        );
        assert_eq!(
            tokens[2][3],
            Token::new(TokenType::Number, "0b001", 3, 24..29)
        );
        assert_eq!(tokens[2][4], Token::new(TokenType::Number, "10", 3, 30..32));
        assert_eq!(
            tokens[2][5],
            Token::new(TokenType::Number, "0o12", 3, 33..37)
        );
        assert_eq!(
            tokens[2][6],
            Token::new(TokenType::Number, "-10", 3, 38..41)
        );
        assert_eq!(
            tokens[2][7],
            Token::new(TokenType::Number, "-0o12", 3, 42..47)
        );

        assert_eq!(
            tokens[3][0],
            Token::new(TokenType::Mnemonic, "JMP", 4, 8..11)
        );
        assert_eq!(
            tokens[3][1],
            Token::new(TokenType::LabelRef, "start", 4, 13..18)
        );

        assert_eq!(tokens[4][0], Token::new(TokenType::Label, "a", 5, 8..9));
        assert_eq!(
            tokens[4][1],
            Token::new(TokenType::Mnemonic, "SUB", 5, 10..13)
        );
        assert_eq!(
            tokens[4][2],
            Token::new(TokenType::MemAddress, "0x2A", 5, 15..19)
        );
        assert_eq!(
            tokens[4][3],
            Token::new(TokenType::Register, "A", 5, 20..21)
        );
        assert_eq!(
            tokens[4][4],
            Token::new(TokenType::Register, "B", 5, 22..23)
        );

        assert_eq!(tokens[5][0], Token::new(TokenType::Label, "b", 6, 8..9));
        assert_eq!(
            tokens[5][1],
            Token::new(TokenType::Mnemonic, "SUB", 6, 10..13)
        );
        assert_eq!(
            tokens[5][2],
            Token::new(TokenType::LabelAddressRef, "label2", 6, 15..21)
        );
        assert_eq!(
            tokens[5][3],
            Token::new(TokenType::Register, "B", 6, 22..23)
        );
        assert_eq!(
            tokens[5][4],
            Token::new(TokenType::Register, "A", 6, 24..25)
        );

        assert_eq!(
            tokens[6][0],
            Token::new(TokenType::Mnemonic, "HALT", 7, 8..12)
        );
    }

    #[test]
    fn test_tokenize_unknown_token() {
        let patterns = create_patterns();
        let input = "INVALID A, 42";

        let result = tokenize(&patterns, input);

        assert!(result.is_err());
    }
}
