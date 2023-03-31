mod lexer;
mod parser;

use lexer::{create_patterns, tokenize};
use parser::parse;
use std::fs;

fn main() {
    let contents = fs::read_to_string("./test.as").expect("Failed to read the file");
    let patterns = create_patterns();
    let tokens = tokenize(&patterns, &contents);
    // let _output = parse(&tokens);

    for token in tokens {
        println!("{:?}", token);
    }
}
