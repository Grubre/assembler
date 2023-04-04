use std::io::read_to_string;
use std::{fs, ops::Deref};
use std::path::PathBuf;

use assembler::{
    error_handler::Error,
    cmdline_args::{get_read_write, Args},
    config::Config,
    lexer::{create_patterns, tokenize},
    parser::parse_all,
    resolver::resolve_all_labels, error_handler::error,
};
use clap::Parser;

fn main() {
    env_logger::init();

    let config_file = "config.cfg";

    let args = Args::parse();

    let (mut input, mut output) = get_read_write(&args).unwrap();

    let config = Config::read_from_file(config_file).unwrap();

    println!("{config:#?}");

    let contents = read_to_string(&mut input).unwrap();
    let patterns = create_patterns();

    let tokens = match tokenize(&patterns, &contents) {
        Ok(tokens) => tokens,
        Err(err) => match err {
            assembler::lexer::TokenizeError::UnknownToken(line_nr, _char_nr, _token) => {
                error(Error {
                    input_file: args.input_file.unwrap().as_path(),
                    line_nr,
                    error_string: "Unknown token"
                });
                panic!()
            },
        },
    };

    // println!("{tokens:#?}");

    let (unresolved, labels) = parse_all(&tokens, &config);

    println!("{unresolved:#?}");
    // println!("{labels:#?}");

    let resolved = resolve_all_labels(&labels, unresolved);

    println!("{resolved:#?}");

    for line in resolved {
        writeln!(&mut output, "{}", line).unwrap();
    }
}
