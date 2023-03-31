mod lexer;
mod parser;
mod config;

use config::InstructionDef;
use lexer::{create_patterns, tokenize};
use parser::parse;
use std::fs;

use crate::config::Config;

fn main() {
    // let contents = fs::read_to_string("./test.as").expect("Failed to read the file");
    // let patterns = create_patterns();
    // let tokens = tokenize(&patterns, &contents);
    // // let _output = parse(&tokens);

    // for token in tokens {
    //     println!("{:?}", token);
    // }

    let v = Config::read_from_file("config.cfg").unwrap();

    println!("{v:#?}");

}
