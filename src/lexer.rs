use regex::Regex;

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
}

#[derive(Debug, PartialEq, Hash)]
pub struct Token {
    pub token_type: TokenType,
    pub content: String,
    pub line_nr: usize,
}

pub fn create_patterns() -> Vec<(TokenType, Regex)> {
    let patterns = vec![
        (
            TokenType::Mnemonic,
            Regex::new(r"(?i)^(NOP|MOV|PUSH|POP|JMP|ADD|SUB|OR|AND|NEG|INV|SHR|SHL|CMP|HALT)")
                .unwrap(),
        ),
        (TokenType::Register, Regex::new(r"^(A|B)").unwrap()),
        (
            TokenType::Number,
            Regex::new(r"^(0x[0-9A-Fa-f]+|0b[01]+|0o[0-7]+|[0-9]+)").unwrap(),
        ),
        (
            TokenType::MemAddress,
            Regex::new(r"^\[(0x[0-9A-Fa-f]+|0b[01]+|0o[0-7]+|[0-9]+)\]").unwrap(),
        ),
        (
            TokenType::Label,
            Regex::new(r"^([a-zA-Z_][a-zA-Z0-9_]*):").unwrap(),
        ),
        (
            TokenType::LabelRef,
            Regex::new(r"^#([a-zA-Z_][a-zA-Z0-9_]*)").unwrap(),
        ),
        (
            TokenType::LabelAddressRef,
            Regex::new(r"^\[#([a-zA-Z_][a-zA-Z0-9_]*)\]").unwrap(),
        ),
        (TokenType::Comment, Regex::new(r"^;.*").unwrap()),
    ];

    patterns
}

#[derive(Debug, PartialEq, Eq)]
pub enum TokenizeError {
    UnknownToken(usize, usize)
}

pub fn tokenize(patterns: &Vec<(TokenType, Regex)>, input: &str) -> Result<Vec<Vec<Token>>, TokenizeError> {
    let mut tokens = Vec::new();

    for (i, line) in input.lines().enumerate() {
        let mut line_ref = line.trim();
        let mut line_tokens = Vec::new();

        while !line_ref.is_empty() {
            let mut matched = false;

            for (token_type, pattern) in patterns {
                if let Some(word) = pattern.captures(line_ref) {
                    let content = word.get(1).unwrap().as_str();
                    line_tokens.push(Token {
                        token_type: token_type.clone(),
                        content: content.to_string(),
                        line_nr: i,
                    });
                    line_ref = line_ref[word.get(0).unwrap().end()..].trim_start();
                    matched = true;
                    break;
                }
            }

            if !matched {
                return Err(TokenizeError::UnknownToken(i + 1, 0));
            }
        }

        if !line_tokens.is_empty() {
            tokens.push(line_tokens)
        }
    }
    Ok(tokens)
}
