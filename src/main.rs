use std::fs;

use assembler::{config::Config, parser::final_parse, lexer::{create_patterns, tokenize}};

fn main() {
    let config = Config::read_from_file("config.cfg").unwrap();

    println!("{config:#?}");
    
    let contents = fs::read_to_string("./test.as").expect("Failed to read the file");
    let patterns = create_patterns();
    let tokens = tokenize(&patterns, &contents);
    for token in &tokens {
        println!("{:?}", token);
    }
    let output = final_parse(&tokens, &config);

    println!("{output:#?}");

}
