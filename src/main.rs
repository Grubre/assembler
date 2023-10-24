use std::error::Error;

use assembler::{
    cmdline_args::Args,
    config::{print_config, Config},
};
use clap::Parser;
use owo_colors::OwoColorize;

trait ConsumeErrorExt<T, E> {
    fn consume_error(self) -> Result<T, ()>;
}

impl<T, E> ConsumeErrorExt<T, E> for Result<T, E>
where
    E: Error + std::fmt::Display,
{
    fn consume_error(self) -> Result<T, ()> {
        match self {
            Ok(value) => Ok(value),
            Err(err) => {
                eprintln!(
                    "{} {} {}",
                    "assembly:".bold(),
                    "fatal error:".red().bold(),
                    err
                );
                Err(())
            }
        }
    }
}

fn main() -> Result<(), ()> {
    let config_file = "config.cfg";

    let args = Args::parse();

    let (mut input, mut output) = Args::get_read_write(&args).consume_error()?;

    let config = Config::read_from_file(config_file).consume_error()?;

    // let contents = read_to_string(&mut input).unwrap();

    // let file_ctx = FileContext::new(args.input_file.as_deref(), &contents);
    //
    // let patterns = create_patterns();
    //
    // let tokens = tokenize(&patterns, &contents)
    //     .map_err(|err| err.throw_all_with_ctx(&file_ctx))
    //     .unwrap();
    //
    // println!("{tokens:#?}");
    //
    // let (unresolved, labels) = parse_all(tokens, &config)
    //     .map_err(|err| err.throw_all_with_ctx(&file_ctx))
    //     .unwrap();
    //
    // println!("{unresolved:#?}");
    // // println!("{labels:#?}");
    //
    // let resolved = resolve_all_labels(&labels, unresolved)
    //     .map_err(|err| err.throw_all_with_ctx(&file_ctx))
    //     .unwrap();
    //
    // //println!("{resolved:#?}");
    //
    // for line in resolved {
    //     writeln!(&mut output, "{}", line).unwrap();
    // }
    Ok(())
}
