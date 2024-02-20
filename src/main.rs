use std::{error::Error, io::read_to_string, io::Write, process::exit};

use assembler::{
    checker::{check_semantics, CheckedLine, CheckedLineCode},
    cmdline_args::Args,
    config::{print_config, Config},
    lexer::Lexer,
    parser::parse,
    resolver::get_resolved_labels,
};
use clap::Parser;
use owo_colors::OwoColorize;

trait ConsumeError<T, E> {
    fn consume_error(self) -> T;
}

fn print_error<E: Error + std::fmt::Display>(error: E) {
    eprintln!(
        "{} {} {}",
        "assembly:".bold(),
        "fatal error:".red().bold(),
        error
    );
}

impl<T, E> ConsumeError<T, E> for Result<T, E>
where
    E: Error + std::fmt::Display,
{
    fn consume_error(self) -> T {
        match self {
            Ok(value) => value,
            Err(err) => {
                print_error(err);
                exit(1);
            }
        }
    }
}

trait ConsumeErrorVec<T, E> {
    fn consume_errors(self) -> Vec<T>;
}

impl<T, E> ConsumeErrorVec<T, E> for Result<Vec<T>, Vec<E>>
where
    E: Error + std::fmt::Display,
{
    fn consume_errors(self) -> Vec<T> {
        let errors = match self {
            Ok(lines) => return lines,
            Err(errs) => errs,
        };
        for err in errors {
            print_error(err);
        }
        exit(1);
    }
}

impl<T, E> ConsumeErrorVec<T, E> for Vec<Result<T, E>>
where
    E: Error + std::fmt::Display,
{
    fn consume_errors(self) -> Vec<T> {
        let mut ts = Vec::new();
        let mut found_error = false;
        for result in self {
            match result {
                Ok(t) => ts.push(t),
                Err(err) => {
                    print_error(err);
                    found_error = true;
                }
            }
        }
        if found_error {
            exit(1);
        }
        ts
    }
}

pub trait ResultSplit<T, E> {
    fn result_split(self) -> Result<Vec<T>, Vec<E>>;
}

impl<T, E, I: Iterator<Item = Result<T, E>>> ResultSplit<T, E> for I {
    fn result_split(self) -> Result<Vec<T>, Vec<E>> {
        let (ok, err): (Vec<_>, Vec<_>) = self.partition(Result::is_ok);

        if err.is_empty() {
            let ok = ok.into_iter().map(|t| t.ok().unwrap()).collect();
            Ok(ok)
        } else {
            let err = err.into_iter().map(|t| t.err().unwrap()).collect();
            Err(err)
        }
    }
}

fn output_bytes_as_text(checked_lines: &[CheckedLine], output: &mut Box<dyn Write>) {
    for checked_line in checked_lines {
        match &checked_line.code {
            assembler::checker::CheckedLineCode::Byte(bytes) => {
                for byte in bytes {
                    output
                        .write_all(format!("{:08b}", byte).as_bytes())
                        .unwrap();
                    output.write_all(&[b'\n']).unwrap();
                }
            }
            assembler::checker::CheckedLineCode::Instruction {
                mnemonic_code,
                operand_codes,
            } => {
                // TODO: Find a sane way to do that
                output
                    .write_all(format!("{:08b}", mnemonic_code).as_bytes())
                    .unwrap();
                output.write_all(&[b'\n']).unwrap();
                for operand_code in operand_codes {
                    output
                        .write_all(format!("{:08b}", operand_code).as_bytes())
                        .unwrap();
                    output.write_all(&[b'\n']).unwrap();
                }
            }
        }
    }
}

fn output_to_binary(checked_lines: &[CheckedLine], output: &mut Box<dyn Write>) {
    let mut output_string = String::new();

    for checked_line in checked_lines {
        match &checked_line.code {
            CheckedLineCode::Byte(bytes) => {
                for byte in bytes {
                    output_string.push(*byte as char);
                }
            }
            CheckedLineCode::Instruction {
                mnemonic_code,
                operand_codes,
            } => {
                output_string.push(*mnemonic_code as char);
                for operand_code in operand_codes {
                    output_string.push(*operand_code as char);
                }
            }
        }
    }

    output.write_all(output_string.as_bytes()).unwrap();
}

fn main() -> Result<(), ()> {
    let args = Args::parse();
    let (mut input, mut output) = Args::get_read_write(&args).consume_error();
    let config_file = args.config_file.unwrap_or("config.cfg".into());

    let config = Config::read_from_file(config_file).consume_error();

    print_config(&config);
    //
    // let contents = read_to_string(&mut input).unwrap();
    // let chars = contents.chars().collect::<Vec<_>>();
    //
    // let tokens = Lexer::new(&chars).collect::<Vec<_>>().consume_errors();
    // let labels = get_resolved_labels(&tokens);
    //
    // let lines = parse(&tokens).consume_errors();
    // let checked_lines = check_semantics(lines, &labels, &config).consume_error();
    //
    // if args.text {
    //     output_bytes_as_text(&checked_lines, &mut output);
    // } else {
    //     output_to_binary(&checked_lines, &mut output);
    // }

    Ok(())
}
