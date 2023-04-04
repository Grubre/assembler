use std::path::Path;

pub struct Error<'a> {
    pub input_file: &'a Path,
    pub line_nr: usize,
    pub char_index: usize,
    pub error_string: &'a str,
}

use owo_colors::OwoColorize;

pub fn throw_error(err: Error) {
    eprintln!(
        "{}:{}:{}: {} {}",
        err.input_file.to_str().unwrap().bold(),
        err.line_nr.bold(),
        err.char_index.bold(),
        "error:".red().bold(),
        err.error_string
    );
    panic!()
}
