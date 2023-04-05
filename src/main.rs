use std::io::read_to_string;

use assembler::{
    cmdline_args::{get_read_write, Args},
    config::Config,
    error::{ErrorGroup, FileContext},
    lexer::{create_patterns, tokenize},
    parser::parse_all,
    resolver::resolve_all_labels,
};
use clap::Parser;

fn main() {
    let config_file = "config.cfg";

    let args = Args::parse();

    let (mut input, mut output) = get_read_write(&args).unwrap();

    let config = Config::read_from_file(config_file).unwrap();

    // println!("{config:#?}");

    let contents = read_to_string(&mut input).unwrap();

    let file_ctx = FileContext::new(args.input_file.as_deref(), &contents);

    let patterns = create_patterns();

    let tokens = tokenize(&patterns, &contents)
        .map_err(|err| err.throw_all_with_ctx(&file_ctx))
        .unwrap();

    println!("{tokens:#?}");

    let (unresolved, labels) = parse_all(tokens, &config)
        .map_err(|err| err.throw_all_with_ctx(&file_ctx))
        .unwrap();

    println!("{unresolved:#?}");
    // println!("{labels:#?}");

    let resolved = resolve_all_labels(&labels, unresolved)
        .map_err(|err| err.throw_all_with_ctx(&file_ctx))
        .unwrap();

    //println!("{resolved:#?}");

    for line in resolved {
        writeln!(&mut output, "{}", line).unwrap();
    }
}
