use std::fs;

use assembler::{config::Config, lexer::{create_patterns, tokenize}, parser::parse_all, resolver::resolve_all_labels};

fn main() {

    //Inicjalizacja loggera
    env_logger::init();

    let config = Config::read_from_file("config.cfg").unwrap();

    println!("{config:#?}");
    
    let contents = fs::read_to_string("./test.as").expect("Failed to read the file");
    let patterns = create_patterns();
    let tokens = tokenize(&patterns, &contents);
    println!("{tokens:#?}");
    let (unresolved, labels) = parse_all(&tokens, &config);
    println!("{unresolved:#?}");
    println!("{labels:#?}");
    let resolved = resolve_all_labels(&labels, unresolved);
    println!("{resolved:#?}")

}
