use std::collections::HashMap;

use crate::token::{Token, TokenType};

pub fn resolve(tokens: &[Token]) -> HashMap<&str, usize> {
    let mut memory_pointer = 0;
    let mut labels: HashMap<&str, usize> = HashMap::new();

    for token in tokens {
        match &token.token_type {
            TokenType::Mnemonic(_) | TokenType::Number(_) | TokenType::LabelRef(_) => {
                memory_pointer += 1
            }
            TokenType::Label(label) => {
                labels.insert(label, memory_pointer);
            }
            _ => {}
        }
    }

    labels
}
