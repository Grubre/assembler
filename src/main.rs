mod lexer;
mod parser;

use std::fs;
use parser::parse;
use lexer::{tokenize, create_patterns};

fn main() {
    let contents = fs::read_to_string("./test.as").expect("Failed to read the file");
    let tokens = tokenize(&create_patterns(), &contents);
    // let _output = parse(&tokens);

    for token in tokens {
        println!("{:?}", token);
    }
}
