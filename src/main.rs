use std::fs;
use std::path::PathBuf;

use assembler::{
    cmdline_args::{get_io_files, Args},
    config::Config,
    lexer::{create_patterns, tokenize},
    parser::parse_all,
    resolver::resolve_all_labels,
};
use clap::Parser;

fn main() {
    env_logger::init();

    let args = Args::parse();

    let (input_file, output_file) = get_io_files(&args);

    println!("Input  file: {}", input_file.display());
    println!("Output file: {}", output_file.display());

    let config = Config::read_from_file(input_file).unwrap();

    // println!("{config:#?}");
    //
    // let contents = fs::read_to_string("./test.as").expect("Failed to read the file");
    // let patterns = create_patterns();
    //
    // let tokens = match tokenize(&patterns, &contents) {
    //     Ok(tokens) => tokens,
    //     Err(err) => match err {
    //         assembler::lexer::TokenizeError::UnknownToken(line_nr, char_nr) => {
    //             eprintln!("Unknown token at line {} ", line_nr);
    //             panic!()
    //         }
    //     },
    // };
    //
    // // println!("{tokens:#?}");
    //
    // let (unresolved, labels) = parse_all(&tokens, &config);
    //
    // // println!("{unresolved:#?}");
    // // println!("{labels:#?}");
    //
    // let resolved = resolve_all_labels(&labels, unresolved);
    //
    // println!("{resolved:#?}")
}
