use std::{
    cmp::{max, min},
    ops::{Add, Range},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Span {
    pub line: usize,
    pub chars: Range<usize>,
}

impl Span {
    pub fn new(line: usize, chars: Range<usize>) -> Self {
        Span { line, chars }
    }
}

impl Add for Span {
    type Output = Span;

    fn add(self, rhs: Self) -> Self::Output {
        let start = min(self.chars.start, rhs.chars.start);
        let end = max(self.chars.end, rhs.chars.end);
        Span::new(rhs.line, start..end)
    }
}

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

// TODO: Remove manual Eq and PartialEq implementation
// TODO: But then the tests don't pass and I don't feel like fixing it now
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
